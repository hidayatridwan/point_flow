<script lang="ts">
	import CalendarIcon from '@lucide/svelte/icons/calendar';
	import TrashIcon from '@lucide/svelte/icons/trash';
	import { parseDate, type DateValue } from '@internationalized/date';
	import { createGoodsReceipt, type GoodsReceipt } from '$lib/api/goods-receipts.js';
	import {
		getPurchaseOrder,
		listPurchaseOrders,
		type PurchaseOrder,
		type PurchaseOrderDetail,
		type PurchaseOrderItem
	} from '$lib/api/purchase-orders.js';
	import { ApiError } from '$lib/api/types.js';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import { Button, buttonVariants } from '$lib/components/ui/button/index.js';
	import Calendar from '$lib/components/ui/calendar/calendar.svelte';
	import * as Drawer from '$lib/components/ui/drawer/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import * as Popover from '$lib/components/ui/popover/index.js';
	import * as Select from '$lib/components/ui/select/index.js';
	import { Spinner } from '$lib/components/ui/spinner/index.js';
	import { Textarea } from '$lib/components/ui/textarea/index.js';
	import { formatDate, formatIdr, formatNumber } from '$lib/formatters.js';
	import { cn } from '$lib/utils.js';
	import { toast } from 'svelte-sonner';
	import { z } from 'zod';
	import { t } from '$lib/i18n/locale.svelte.js';

	const normalizeStringInput = (value: unknown) => {
		if (value === null || value === undefined) return '';
		return String(value);
	};

	type GoodsReceiptFormItem = {
		client_id: string;
		purchase_order_item_id: string;
		product_name: string;
		product_sku: string;
		unit: string;
		quantity_ordered: number;
		quantity_received_to_date: number;
		remaining_quantity: number;
		quantity_received: string;
		unit_cost: string;
		margin_percent: string;
	};

	type GoodsReceiptForm = {
		purchase_order_id: string;
		received_date: string;
		notes: string;
		items: GoodsReceiptFormItem[];
	};

	const requiredIdSchema = (label: string) =>
		z.preprocess(
			normalizeStringInput,
			z
				.string()
				.trim()
				.min(1, `${label} is required.`)
				.refine((value) => Number.isInteger(Number(value)) && Number(value) > 0, {
					message: `${label} must be selected.`
				})
		);

	const positiveNumberSchema = (label: string) =>
		z.preprocess(
			normalizeStringInput,
			z
				.string()
				.trim()
				.min(1, `${label} is required.`)
				.refine((value) => Number.isFinite(Number(value)), {
					message: `${label} must be a valid number.`
				})
				.refine((value) => Number(value) > 0, {
					message: `${label} must be greater than zero.`
				})
		);

	const nonNegativeNumberSchema = (label: string) =>
		z.preprocess(
			normalizeStringInput,
			z
				.string()
				.trim()
				.min(1, `${label} is required.`)
				.refine((value) => Number.isFinite(Number(value)), {
					message: `${label} must be a valid number.`
				})
				.refine((value) => Number(value) >= 0, {
					message: `${label} must be zero or greater.`
				})
		);

	// Opsional: kosong = biarkan selling price SKU tidak berubah saat posting.
	const optionalMarginSchema = (label: string) =>
		z.preprocess(
			normalizeStringInput,
			z
				.string()
				.trim()
				.refine((value) => value === '' || Number.isFinite(Number(value)), {
					message: `${label} must be a valid number.`
				})
				.refine((value) => value === '' || Number(value) >= 0, {
					message: `${label} must be zero or greater.`
				})
				.refine((value) => value === '' || /^\d+(\.\d{1,2})?$/.test(value), {
					message: `${label} must have at most 2 decimal places.`
				})
		);

	const goodsReceiptItemSchema = z.object({
		client_id: z.string(),
		purchase_order_item_id: requiredIdSchema('Purchase order item'),
		product_name: z.string(),
		product_sku: z.string(),
		unit: z.string(),
		quantity_ordered: z.number(),
		quantity_received_to_date: z.number(),
		remaining_quantity: z.number(),
		quantity_received: positiveNumberSchema('Received quantity'),
		unit_cost: nonNegativeNumberSchema('Unit cost'),
		margin_percent: optionalMarginSchema('Profit margin')
	});

	const createGoodsReceiptSchema = z
		.object({
			purchase_order_id: requiredIdSchema('Purchase order'),
			received_date: z.string().trim().min(1, 'Received date is required.'),
			notes: z.string().trim().max(500, 'Notes must be at most 500 characters.'),
			items: z.array(goodsReceiptItemSchema).min(1, 'At least one line item is required.')
		})
		.superRefine((data, context) => {
			data.items.forEach((item, index) => {
				if (Number(item.quantity_received) > item.remaining_quantity) {
					context.addIssue({
						code: 'custom',
						path: ['items', index, 'quantity_received'],
						message: 'Received quantity must not exceed remaining quantity.'
					});
				}
			});
		});

	let {
		open = $bindable(false),
		onCreated
	}: {
		open: boolean;
		onCreated?: (goodsReceipt: GoodsReceipt) => void | Promise<void>;
	} = $props();

	let purchaseOrders = $state<PurchaseOrder[]>([]);
	let selectedPurchaseOrder = $state<PurchaseOrderDetail | null>(null);
	let lineItemSequence = 0;
	const initialReceivedDate = getTodayDate();
	let receivedDateValue = $state<DateValue | undefined>(dateStringToDateValue(initialReceivedDate));
	let isReceivedDateOpen = $state(false);
	let formData = $state<GoodsReceiptForm>(getDefaultFormData(initialReceivedDate));
	let fieldErrors = $state<Record<string, string>>({});
	let isLoadingOptions = $state(false);
	let isLoadingPurchaseOrder = $state(false);
	let isSubmitting = $state(false);
	let errorMessage = $state<string | null>(null);
	let optionsErrorMessage = $state<string | null>(null);
	let latestOptionsRequestId = 0;
	let latestPurchaseOrderRequestId = 0;

	const totalAmount = $derived(
		formData.items.reduce((total, item) => total + getLineTotal(item), 0)
	);

	$effect(() => {
		if (!open) return;
		resetFormData();
		fieldErrors = {};
		errorMessage = null;
		optionsErrorMessage = null;
		void loadOptions();
	});

	$effect(() => {
		const receivedDate = dateValueToDateString(receivedDateValue);
		if (formData.received_date !== receivedDate) {
			formData.received_date = receivedDate;
		}
		if (receivedDateValue) {
			isReceivedDateOpen = false;
		}
	});

	async function loadOptions() {
		const requestId = ++latestOptionsRequestId;
		isLoadingOptions = true;
		optionsErrorMessage = null;

		try {
			const result = await listPurchaseOrders({
				limit: 100,
				status: ['ordered', 'partially_received']
			});
			if (requestId !== latestOptionsRequestId) return;
			purchaseOrders = result.purchaseOrders;
		} catch (error) {
			if (requestId !== latestOptionsRequestId) return;
			purchaseOrders = [];
			optionsErrorMessage = getErrorMessage(error, 'Failed to load purchase order options.');
		} finally {
			if (requestId === latestOptionsRequestId) {
				isLoadingOptions = false;
			}
		}
	}

	async function selectPurchaseOrder(value: string) {
		formData.purchase_order_id = value;
		selectedPurchaseOrder = null;
		formData.items = [];
		fieldErrors = {};
		if (!value) return;

		const purchaseOrderId = Number(value);
		if (!Number.isFinite(purchaseOrderId)) return;

		const requestId = ++latestPurchaseOrderRequestId;
		isLoadingPurchaseOrder = true;
		optionsErrorMessage = null;

		try {
			const purchaseOrder = await getPurchaseOrder(purchaseOrderId);
			if (requestId !== latestPurchaseOrderRequestId) return;
			selectedPurchaseOrder = purchaseOrder;
			formData.items = purchaseOrder.items
				.filter((item) => getRemainingQuantity(item) > 0)
				.map(createItemFromPurchaseOrderItem);

			if (!formData.items.length) {
				optionsErrorMessage = 'Selected purchase order has no remaining items to receive.';
			}
		} catch (error) {
			if (requestId !== latestPurchaseOrderRequestId) return;
			selectedPurchaseOrder = null;
			optionsErrorMessage = getErrorMessage(error, 'Failed to load purchase order details.');
		} finally {
			if (requestId === latestPurchaseOrderRequestId) {
				isLoadingPurchaseOrder = false;
			}
		}
	}

	async function handleSubmit(event: SubmitEvent) {
		event.preventDefault();
		const result = createGoodsReceiptSchema.safeParse(formData);

		if (!result.success) {
			fieldErrors = getValidationErrors(result.error);
			return;
		}

		isSubmitting = true;
		errorMessage = null;
		fieldErrors = {};

		try {
			const createdGoodsReceipt = await createGoodsReceipt({
				purchase_order_id: Number(result.data.purchase_order_id),
				received_at: dateToUtcDateTime(result.data.received_date),
				notes: emptyToNull(result.data.notes),
				items: result.data.items.map((item) => ({
					purchase_order_item_id: Number(item.purchase_order_item_id),
					quantity_received: Number(item.quantity_received),
					unit_cost: Number(item.unit_cost),
					margin_percent: item.margin_percent.trim() === '' ? null : Number(item.margin_percent)
				}))
			});
			await onCreated?.(createdGoodsReceipt);
			toast.success(t('procurement.goodsReceipts.createdToast'), {
				description: `${createdGoodsReceipt.gr_no} was created as draft.`
			});
			open = false;
		} catch (error) {
			errorMessage = getErrorMessage(error, t('procurement.goodsReceipts.errorCreate'));
		} finally {
			isSubmitting = false;
		}
	}

	function removeLineItem(clientId: string) {
		if (formData.items.length === 1) return;
		formData.items = formData.items.filter((item) => item.client_id !== clientId);
	}

	function createItemFromPurchaseOrderItem(item: PurchaseOrderItem): GoodsReceiptFormItem {
		lineItemSequence += 1;
		const remainingQuantity = getRemainingQuantity(item);
		return {
			client_id: `goods-receipt-item-${lineItemSequence}`,
			purchase_order_item_id: String(item.id),
			product_name: item.product_name,
			product_sku: item.product_sku,
			unit: item.unit,
			quantity_ordered: item.quantity_ordered,
			quantity_received_to_date: item.quantity_received,
			remaining_quantity: remainingQuantity,
			quantity_received: String(remainingQuantity),
			unit_cost: String(item.unit_cost),
			margin_percent: ''
		};
	}

	function getRemainingQuantity(item: PurchaseOrderItem) {
		return Math.max(0, item.quantity_ordered - item.quantity_received);
	}

	function getPurchaseOrderLabel(purchaseOrder: PurchaseOrder) {
		return `${purchaseOrder.po_no} - ${purchaseOrder.supplier_name}`;
	}

	function getLineTotal(item: GoodsReceiptFormItem) {
		return toNumber(item.quantity_received) * toNumber(item.unit_cost);
	}

	function getNewSellingPrice(item: GoodsReceiptFormItem) {
		if (item.margin_percent.trim() === '') return null;
		const margin = toNumber(item.margin_percent);
		const unitCost = toNumber(item.unit_cost);
		if (!Number.isFinite(margin) || !Number.isFinite(unitCost)) return null;
		return Math.round(unitCost * (1 + margin / 100) * 100) / 100;
	}

	function toNumber(value: string) {
		const number = Number(value);
		return Number.isFinite(number) ? number : 0;
	}

	function resetFormData() {
		const receivedDate = getTodayDate();
		receivedDateValue = dateStringToDateValue(receivedDate);
		isReceivedDateOpen = false;
		selectedPurchaseOrder = null;
		formData = getDefaultFormData(receivedDate);
	}

	function getDefaultFormData(receivedDate = getTodayDate()): GoodsReceiptForm {
		return {
			purchase_order_id: '',
			received_date: receivedDate,
			notes: '',
			items: []
		};
	}

	function getTodayDate() {
		const date = new Date();
		const year = date.getFullYear();
		const month = String(date.getMonth() + 1).padStart(2, '0');
		const day = String(date.getDate()).padStart(2, '0');
		return `${year}-${month}-${day}`;
	}

	function dateStringToDateValue(value: string): DateValue | undefined {
		if (!value) return undefined;

		try {
			return parseDate(value);
		} catch {
			return undefined;
		}
	}

	function dateValueToDateString(value: DateValue | undefined) {
		return value?.toString() ?? '';
	}

	function dateToUtcDateTime(value: string) {
		return `${value}T00:00:00Z`;
	}

	function getValidationErrors(error: z.ZodError<GoodsReceiptForm>) {
		const errors: Record<string, string> = {};
		for (const issue of error.issues) {
			const key = issue.path.map(String).join('.');
			if (!errors[key]) errors[key] = issue.message;
		}
		return errors;
	}

	function getFieldError(path: string) {
		return fieldErrors[path] ?? null;
	}

	function emptyToNull(value: string) {
		const trimmed = value.trim();
		return trimmed ? trimmed : null;
	}

	function getErrorMessage(error: unknown, fallback: string) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return fallback;
	}
</script>

<Drawer.Root bind:open direction="bottom">
	<Drawer.Content class="max-h-[90vh] overflow-hidden p-0">
		<form
			class="mx-auto flex max-h-[90vh] min-h-0 w-full max-w-5xl flex-col"
			onsubmit={handleSubmit}
		>
			<Drawer.Header
				class="shrink-0 border-b px-6 py-5 text-left group-data-[vaul-drawer-direction=bottom]/drawer-content:text-left"
			>
				<Drawer.Title>{t('procurement.goodsReceipts.addTitle')}</Drawer.Title>
				<Drawer.Description>{t('procurement.goodsReceipts.addDesc')}</Drawer.Description>
			</Drawer.Header>

			<div class="flex-1 overflow-y-auto px-6 py-5">
				<div class="grid gap-5">
					<div class="grid grid-cols-1 gap-x-4 gap-y-5 md:grid-cols-2">
						<div class="space-y-2">
							<Label for="goods-receipt-purchase-order" class="whitespace-nowrap">
								{t('procurement.purchaseOrders.title')}
							</Label>
							<Select.Root
								allowDeselect={false}
								type="single"
								value={formData.purchase_order_id}
								disabled={isSubmitting || isLoadingOptions}
								onValueChange={(value) => {
									if (typeof value === 'string') {
										void selectPurchaseOrder(value);
									}
								}}
							>
								<Select.Trigger id="goods-receipt-purchase-order" class="w-full">
									<Select.Value placeholder={t('procurement.goodsReceipts.selectOrderedPo')} />
								</Select.Trigger>
								<Select.Content>
									{#each purchaseOrders as purchaseOrder (purchaseOrder.id)}
										<Select.Item
											value={String(purchaseOrder.id)}
											label={getPurchaseOrderLabel(purchaseOrder)}
										>
											{getPurchaseOrderLabel(purchaseOrder)}
										</Select.Item>
									{/each}
								</Select.Content>
							</Select.Root>
							{#if getFieldError('purchase_order_id')}
								<p class="text-sm text-destructive">{getFieldError('purchase_order_id')}</p>
							{/if}
						</div>

						<div class="space-y-2">
							<Label for="goods-receipt-received-date" class="whitespace-nowrap">
								{t('procurement.goodsReceipts.receivedDate')}
							</Label>
							<Popover.Root bind:open={isReceivedDateOpen}>
								<Popover.Trigger
									id="goods-receipt-received-date"
									class={cn(
										buttonVariants({ variant: 'outline' }),
										'h-9 w-full justify-start gap-2 px-3 text-left font-normal',
										!formData.received_date && 'text-muted-foreground'
									)}
									disabled={isSubmitting}
								>
									<CalendarIcon class="size-4" />
									<span class="truncate">
										{formData.received_date
											? formatDate(formData.received_date)
											: t('procurement.goodsReceipts.selectReceivedDate')}
									</span>
								</Popover.Trigger>
								<Popover.Content align="start" class="w-auto p-0">
									<Calendar
										type="single"
										bind:value={receivedDateValue}
										class="rounded-md border-0 shadow-none"
									/>
								</Popover.Content>
							</Popover.Root>
							{#if getFieldError('received_date')}
								<p class="text-sm text-destructive">{getFieldError('received_date')}</p>
							{/if}
						</div>

						<div class="space-y-2 md:col-span-2">
							<Label for="goods-receipt-notes" class="whitespace-nowrap">{t('common.notes')}</Label>
							<Textarea
								id="goods-receipt-notes"
								placeholder={t('procurement.goodsReceipts.notesPlaceholder')}
								class="min-h-9 resize-none"
								disabled={isSubmitting}
								bind:value={formData.notes}
							/>
							{#if getFieldError('notes')}
								<p class="text-sm text-destructive">{getFieldError('notes')}</p>
							{/if}
						</div>
					</div>

					{#if selectedPurchaseOrder}
						<div class="grid grid-cols-1 gap-3 rounded-md border bg-muted/10 p-4 sm:grid-cols-3">
							<div>
								<p class="text-xs text-muted-foreground">{t('procurement.goodsReceipts.poNo')}</p>
								<p class="text-sm font-medium">{selectedPurchaseOrder.po_no}</p>
							</div>
							<div>
								<p class="text-xs text-muted-foreground">
									{t('procurement.purchaseOrders.supplier')}
								</p>
								<p class="text-sm font-medium">{selectedPurchaseOrder.supplier_name}</p>
							</div>
							<div>
								<p class="text-xs text-muted-foreground">
									{t('procurement.purchaseOrders.expectedDate')}
								</p>
								<p class="text-sm font-medium">{formatDate(selectedPurchaseOrder.expected_date)}</p>
							</div>
						</div>
					{/if}

					<div class="flex flex-col gap-3">
						<div>
							<h3 class="text-sm font-medium">{t('procurement.goodsReceipts.receivedItems')}</h3>
							<p class="text-sm text-muted-foreground">
								{t('procurement.goodsReceipts.receivedItemsDesc')}
							</p>
						</div>

						{#if getFieldError('items')}
							<ErrorAlert
								title={t('procurement.goodsReceipts.invalidItems')}
								message={getFieldError('items') ?? ''}
							/>
						{/if}

						{#if isLoadingPurchaseOrder}
							<div class="rounded-md border p-4 text-sm text-muted-foreground">
								{t('procurement.goodsReceipts.loadingItems')}
							</div>
						{:else if formData.items.length > 0}
							<div class="grid gap-3">
								{#each formData.items as item, index (item.client_id)}
									<div class="grid gap-4 rounded-md border bg-muted/10 p-4">
										<div class="flex items-start justify-between gap-3">
											<div>
												<p class="text-sm font-medium">{item.product_name}</p>
												<p class="text-xs text-muted-foreground">
													{item.product_sku} - {item.unit}
												</p>
											</div>
											<Button
												type="button"
												variant="ghost"
												size="icon-sm"
												class="size-8 text-destructive hover:bg-destructive/10 hover:text-destructive"
												disabled={isSubmitting || formData.items.length === 1}
												onclick={() => removeLineItem(item.client_id)}
												title={t('common.removeItem')}
											>
												<TrashIcon />
												<span class="sr-only">{t('common.removeItem')}</span>
											</Button>
										</div>

										<div class="grid grid-cols-1 gap-x-4 gap-y-5 md:grid-cols-2 lg:grid-cols-5">
											<div class="space-y-1 rounded-md bg-background p-3">
												<p class="text-xs text-muted-foreground">
													{t('procurement.goodsReceipts.ordered')}
												</p>
												<p class="text-sm font-medium">{formatNumber(item.quantity_ordered)}</p>
											</div>
											<div class="space-y-1 rounded-md bg-background p-3">
												<p class="text-xs text-muted-foreground">
													{t('procurement.goodsReceipts.received')}
												</p>
												<p class="text-sm font-medium">
													{formatNumber(item.quantity_received_to_date)}
												</p>
											</div>
											<div class="space-y-2">
												<Label
													for={`goods-receipt-item-quantity-${item.client_id}`}
													class="whitespace-nowrap"
												>
													{t('procurement.goodsReceipts.receiveQty')}
												</Label>
												<Input
													id={`goods-receipt-item-quantity-${item.client_id}`}
													type="number"
													min="0.001"
													max={item.remaining_quantity}
													step="0.001"
													disabled={isSubmitting}
													bind:value={item.quantity_received}
												/>
												<p class="text-xs text-muted-foreground">
													{t('procurement.goodsReceipts.remaining')}
													{formatNumber(item.remaining_quantity)}
												</p>
												{#if getFieldError(`items.${index}.quantity_received`)}
													<p class="text-sm text-destructive">
														{getFieldError(`items.${index}.quantity_received`)}
													</p>
												{/if}
											</div>
											<div class="space-y-2">
												<Label
													for={`goods-receipt-item-unit-cost-${item.client_id}`}
													class="whitespace-nowrap"
												>
													{t('procurement.goodsReceipts.unitCost')}
												</Label>
												<Input
													id={`goods-receipt-item-unit-cost-${item.client_id}`}
													type="number"
													min="0"
													step="1"
													disabled={isSubmitting}
													bind:value={item.unit_cost}
												/>
												<p class="text-xs text-muted-foreground">
													{t('common.lineTotal')}
													{formatIdr(getLineTotal(item))}
												</p>
												{#if getFieldError(`items.${index}.unit_cost`)}
													<p class="text-sm text-destructive">
														{getFieldError(`items.${index}.unit_cost`)}
													</p>
												{/if}
											</div>
											<div class="space-y-2">
												<Label
													for={`goods-receipt-item-margin-${item.client_id}`}
													class="whitespace-nowrap"
												>
													{t('procurement.goodsReceipts.marginLabel')}
												</Label>
												<Input
													id={`goods-receipt-item-margin-${item.client_id}`}
													type="number"
													min="0"
													step="0.01"
													placeholder={t('procurement.goodsReceipts.marginPlaceholder')}
													disabled={isSubmitting}
													bind:value={item.margin_percent}
												/>
												{#if getNewSellingPrice(item) !== null}
													<p class="text-xs text-muted-foreground">
														{t('procurement.goodsReceipts.newSellingPrice')}:
														{formatIdr(getNewSellingPrice(item) ?? 0)}
													</p>
												{/if}
												{#if getFieldError(`items.${index}.margin_percent`)}
													<p class="text-sm text-destructive">
														{getFieldError(`items.${index}.margin_percent`)}
													</p>
												{/if}
											</div>
										</div>
									</div>
								{/each}
							</div>
						{:else}
							<div class="rounded-md border p-4 text-sm text-muted-foreground">
								{t('procurement.goodsReceipts.selectPoFirst')}
							</div>
						{/if}
					</div>

					<div class="flex justify-end border-t pt-4">
						<div class="text-right">
							<p class="text-sm text-muted-foreground">
								{t('procurement.goodsReceipts.totalReceived')}
							</p>
							<p class="text-lg font-semibold">{formatIdr(totalAmount)}</p>
						</div>
					</div>

					<div class="grid gap-3">
						{#if optionsErrorMessage}
							<ErrorAlert
								title={t('procurement.goodsReceipts.errorLoadOptions')}
								message={optionsErrorMessage}
							/>
						{/if}

						{#if errorMessage}
							<ErrorAlert
								title={t('procurement.goodsReceipts.errorCreate')}
								message={errorMessage}
							/>
						{/if}
					</div>
				</div>
			</div>

			<Drawer.Footer class="shrink-0 border-t bg-muted/30 px-6 py-4 sm:flex-row sm:justify-end">
				<Drawer.Close
					type="button"
					class={cn(buttonVariants({ variant: 'outline' }))}
					disabled={isSubmitting}
				>
					{t('common.cancel')}
				</Drawer.Close>
				<Button type="submit" disabled={isSubmitting || isLoadingOptions || isLoadingPurchaseOrder}>
					{#if isSubmitting}
						<Spinner />
					{/if}
					{isSubmitting
						? t('procurement.goodsReceipts.creating')
						: t('procurement.goodsReceipts.createBtn')}
				</Button>
			</Drawer.Footer>
		</form>
	</Drawer.Content>
</Drawer.Root>
