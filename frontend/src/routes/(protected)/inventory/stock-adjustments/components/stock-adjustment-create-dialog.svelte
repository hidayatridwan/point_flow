<script lang="ts">
	import CalendarIcon from '@lucide/svelte/icons/calendar';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import TrashIcon from '@lucide/svelte/icons/trash';
	import { parseDate, type DateValue } from '@internationalized/date';
	import { listProductSkus, type ProductSku } from '$lib/api/product-skus.js';
	import { createStockAdjustment, type StockAdjustment } from '$lib/api/stock-adjustments.js';
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
	import { formatDate, formatNumber } from '$lib/formatters.js';
	import { cn } from '$lib/utils.js';
	import { toast } from 'svelte-sonner';
	import { z } from 'zod';
	import { t } from '$lib/i18n/locale.svelte.js';

	const normalizeStringInput = (value: unknown) => {
		if (value === null || value === undefined) return '';
		return String(value);
	};

	type StockAdjustmentFormItem = {
		client_id: string;
		product_sku_id: string;
		product_name: string;
		product_sku: string;
		unit: string;
		stock_before: number;
		stock_after: string;
		notes: string;
	};

	type StockAdjustmentForm = {
		adjusted_date: string;
		reason: string;
		notes: string;
		items: StockAdjustmentFormItem[];
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

	const stockAdjustmentItemSchema = z.object({
		client_id: z.string(),
		product_sku_id: requiredIdSchema('Product SKU'),
		product_name: z.string(),
		product_sku: z.string(),
		unit: z.string(),
		stock_before: z.number(),
		stock_after: nonNegativeNumberSchema('Stock after'),
		notes: z.string().trim().max(300, 'Item notes must be at most 300 characters.')
	});

	const createStockAdjustmentSchema = z
		.object({
			adjusted_date: z.string().trim().min(1, 'Adjusted date is required.'),
			reason: z.string().trim().min(1, 'Reason is required.').max(120, 'Reason is too long.'),
			notes: z.string().trim().max(500, 'Notes must be at most 500 characters.'),
			items: z.array(stockAdjustmentItemSchema).min(1, 'At least one line item is required.')
		})
		.superRefine((data, context) => {
			const selectedSkuIds = data.items.map((item) => item.product_sku_id).filter(Boolean);
			if (new Set(selectedSkuIds).size !== selectedSkuIds.length) {
				context.addIssue({
					code: 'custom',
					path: ['items'],
					message: 'Each line item must use a unique product SKU.'
				});
			}

			data.items.forEach((item, index) => {
				if (Number(item.stock_after) === item.stock_before) {
					context.addIssue({
						code: 'custom',
						path: ['items', index, 'stock_after'],
						message: 'Stock after must be different from current stock.'
					});
				}
			});
		});

	let {
		open = $bindable(false),
		onCreated
	}: {
		open: boolean;
		onCreated?: (stockAdjustment: StockAdjustment) => void | Promise<void>;
	} = $props();

	let productSkus = $state<ProductSku[]>([]);
	let lineItemSequence = 0;
	const initialAdjustedDate = getTodayDate();
	let adjustedDateValue = $state<DateValue | undefined>(dateStringToDateValue(initialAdjustedDate));
	let isAdjustedDateOpen = $state(false);
	let formData = $state<StockAdjustmentForm>(getDefaultFormData(initialAdjustedDate));
	let fieldErrors = $state<Record<string, string>>({});
	let isLoadingOptions = $state(false);
	let isSubmitting = $state(false);
	let errorMessage = $state<string | null>(null);
	let optionsErrorMessage = $state<string | null>(null);
	let latestOptionsRequestId = 0;

	const totalDelta = $derived(
		formData.items.reduce((total, item) => total + getItemDelta(item), 0)
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
		const adjustedDate = dateValueToDateString(adjustedDateValue);
		if (formData.adjusted_date !== adjustedDate) {
			formData.adjusted_date = adjustedDate;
		}
		if (adjustedDateValue) {
			isAdjustedDateOpen = false;
		}
	});

	async function loadOptions() {
		const requestId = ++latestOptionsRequestId;
		isLoadingOptions = true;
		optionsErrorMessage = null;

		try {
			const result = await listProductSkus({ limit: 100 });
			if (requestId !== latestOptionsRequestId) return;
			productSkus = result.productSkus.filter((productSku) => productSku.is_active);
		} catch (error) {
			if (requestId !== latestOptionsRequestId) return;
			productSkus = [];
			optionsErrorMessage = getErrorMessage(error, 'Failed to load product SKU options.');
		} finally {
			if (requestId === latestOptionsRequestId) {
				isLoadingOptions = false;
			}
		}
	}

	async function handleSubmit(event: SubmitEvent) {
		event.preventDefault();
		const result = createStockAdjustmentSchema.safeParse(formData);

		if (!result.success) {
			fieldErrors = getValidationErrors(result.error);
			return;
		}

		isSubmitting = true;
		errorMessage = null;
		fieldErrors = {};

		try {
			const createdStockAdjustment = await createStockAdjustment({
				adjusted_at: dateToUtcDateTime(result.data.adjusted_date),
				reason: result.data.reason.trim(),
				notes: emptyToNull(result.data.notes),
				items: result.data.items.map((item) => ({
					product_sku_id: Number(item.product_sku_id),
					stock_after: Number(item.stock_after),
					notes: emptyToNull(item.notes)
				}))
			});
			await onCreated?.(createdStockAdjustment);
			toast.success(t('inventory.stockAdjustments.createdToast'), {
				description: `${createdStockAdjustment.adjustment_no} was created as draft.`
			});
			open = false;
		} catch (error) {
			errorMessage = getErrorMessage(error, t('inventory.stockAdjustments.errorCreate'));
		} finally {
			isSubmitting = false;
		}
	}

	function addLineItem() {
		formData.items = [...formData.items, createEmptyItem()];
	}

	function removeLineItem(clientId: string) {
		if (formData.items.length === 1) return;
		formData.items = formData.items.filter((item) => item.client_id !== clientId);
	}

	function updateLineItemProductSku(clientId: string, value: string) {
		const productSku = productSkus.find((item) => String(item.id) === value);
		formData.items = formData.items.map((item) => {
			if (item.client_id !== clientId) return item;
			if (!productSku) {
				return {
					...item,
					product_sku_id: '',
					product_name: '',
					product_sku: '',
					unit: '',
					stock_before: 0,
					stock_after: '',
					notes: item.notes
				};
			}

			return {
				...item,
				product_sku_id: String(productSku.id),
				product_name: productSku.product_name,
				product_sku: productSku.sku,
				unit: productSku.unit_code,
				stock_before: productSku.stock,
				stock_after: String(productSku.stock),
				notes: item.notes
			};
		});
	}

	function createEmptyItem(): StockAdjustmentFormItem {
		lineItemSequence += 1;
		return {
			client_id: `stock-adjustment-item-${lineItemSequence}`,
			product_sku_id: '',
			product_name: '',
			product_sku: '',
			unit: '',
			stock_before: 0,
			stock_after: '',
			notes: ''
		};
	}

	function getProductSkuLabel(productSku: ProductSku) {
		return `${productSku.product_name} - ${productSku.sku}`;
	}

	function isProductSkuAvailableForItem(productSkuId: number, clientId: string) {
		const value = String(productSkuId);
		return !formData.items.some(
			(item) => item.client_id !== clientId && item.product_sku_id === value
		);
	}

	function getItemDelta(item: StockAdjustmentFormItem) {
		return toNumber(item.stock_after) - item.stock_before;
	}

	function formatSignedNumber(value: number) {
		const formattedValue = formatNumber(Math.abs(value));
		if (value > 0) return `+${formattedValue}`;
		if (value < 0) return `-${formattedValue}`;
		return formattedValue;
	}

	function getDeltaClass(value: number) {
		if (value > 0) return 'text-emerald-600 dark:text-emerald-400';
		if (value < 0) return 'text-rose-600 dark:text-rose-400';
		return 'text-muted-foreground';
	}

	function toNumber(value: string) {
		const number = Number(value);
		return Number.isFinite(number) ? number : 0;
	}

	function resetFormData() {
		const adjustedDate = getTodayDate();
		adjustedDateValue = dateStringToDateValue(adjustedDate);
		isAdjustedDateOpen = false;
		formData = getDefaultFormData(adjustedDate);
	}

	function getDefaultFormData(adjustedDate = getTodayDate()): StockAdjustmentForm {
		return {
			adjusted_date: adjustedDate,
			reason: '',
			notes: '',
			items: [createEmptyItem()]
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

	function getValidationErrors(error: z.ZodError) {
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
				<Drawer.Title>{t('inventory.stockAdjustments.addTitle')}</Drawer.Title>
				<Drawer.Description>{t('inventory.stockAdjustments.addDesc')}</Drawer.Description>
			</Drawer.Header>

			<div class="flex-1 overflow-y-auto px-6 py-5">
				<div class="grid gap-5">
					<div class="grid grid-cols-1 gap-x-4 gap-y-5 md:grid-cols-2">
						<div class="space-y-2">
							<Label for="stock-adjustment-adjusted-date" class="whitespace-nowrap">
								{t('inventory.stockAdjustments.adjustedDate')}
							</Label>
							<Popover.Root bind:open={isAdjustedDateOpen}>
								<Popover.Trigger
									id="stock-adjustment-adjusted-date"
									class={cn(
										buttonVariants({ variant: 'outline' }),
										'h-9 w-full justify-start gap-2 px-3 text-left font-normal',
										!formData.adjusted_date && 'text-muted-foreground'
									)}
									disabled={isSubmitting}
								>
									<CalendarIcon class="size-4" />
									<span class="truncate">
										{formData.adjusted_date
											? formatDate(formData.adjusted_date)
											: 'Select adjusted date'}
									</span>
								</Popover.Trigger>
								<Popover.Content align="start" class="w-auto p-0">
									<Calendar
										type="single"
										bind:value={adjustedDateValue}
										class="rounded-md border-0 shadow-none"
									/>
								</Popover.Content>
							</Popover.Root>
							{#if getFieldError('adjusted_date')}
								<p class="text-sm text-destructive">{getFieldError('adjusted_date')}</p>
							{/if}
						</div>

						<div class="space-y-2">
							<Label for="stock-adjustment-reason" class="whitespace-nowrap"
								>{t('inventory.stockAdjustments.reason')}</Label
							>
							<Input
								id="stock-adjustment-reason"
								placeholder={t('inventory.stockAdjustments.reasonPlaceholder')}
								disabled={isSubmitting}
								bind:value={formData.reason}
							/>
							{#if getFieldError('reason')}
								<p class="text-sm text-destructive">{getFieldError('reason')}</p>
							{/if}
						</div>

						<div class="space-y-2 md:col-span-2">
							<Label for="stock-adjustment-notes" class="whitespace-nowrap"
								>{t('common.notes')}</Label
							>
							<Textarea
								id="stock-adjustment-notes"
								placeholder={t('inventory.stockAdjustments.notesPlaceholder')}
								class="min-h-9 resize-none"
								disabled={isSubmitting}
								bind:value={formData.notes}
							/>
							{#if getFieldError('notes')}
								<p class="text-sm text-destructive">{getFieldError('notes')}</p>
							{/if}
						</div>
					</div>

					<div class="flex flex-col gap-3">
						<div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
							<div>
								<h3 class="text-sm font-medium">
									{t('inventory.stockAdjustments.adjustedItems')}
								</h3>
								<p class="text-sm text-muted-foreground">
									{t('inventory.stockAdjustments.adjustedItemsDesc')}
								</p>
							</div>
							<Button
								type="button"
								variant="outline"
								class="w-full sm:w-auto"
								disabled={isSubmitting || isLoadingOptions}
								onclick={addLineItem}
							>
								<PlusIcon />
								{t('common.addItem')}
							</Button>
						</div>

						{#if getFieldError('items')}
							<ErrorAlert
								title={t('inventory.stockAdjustments.invalidItems')}
								message={getFieldError('items') ?? ''}
							/>
						{/if}

						<div class="grid gap-3">
							{#each formData.items as item, index (item.client_id)}
								<div class="grid gap-4 rounded-md border bg-muted/10 p-4">
									<div class="flex items-start justify-between gap-3">
										<div>
											<p class="text-sm font-medium">Item {index + 1}</p>
											<p class="text-xs text-muted-foreground">
												{item.product_name || t('common.selectProductSku')}
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

									<div class="grid grid-cols-1 gap-x-4 gap-y-5 md:grid-cols-2 lg:grid-cols-4">
										<div class="space-y-2 lg:col-span-2">
											<Label
												for={`stock-adjustment-item-sku-${item.client_id}`}
												class="whitespace-nowrap"
											>
												{t('inventory.stockAdjustments.productSku')}
											</Label>
											<Select.Root
												allowDeselect={false}
												type="single"
												value={item.product_sku_id}
												disabled={isSubmitting || isLoadingOptions}
												onValueChange={(value) => {
													if (typeof value === 'string') {
														updateLineItemProductSku(item.client_id, value);
													}
												}}
											>
												<Select.Trigger
													id={`stock-adjustment-item-sku-${item.client_id}`}
													class="w-full"
												>
													<Select.Value placeholder={t('common.selectProductSku')} />
												</Select.Trigger>
												<Select.Content>
													{#each productSkus.filter( (productSku) => isProductSkuAvailableForItem(productSku.id, item.client_id) ) as productSku (productSku.id)}
														<Select.Item
															value={String(productSku.id)}
															label={getProductSkuLabel(productSku)}
														>
															{getProductSkuLabel(productSku)}
														</Select.Item>
													{/each}
												</Select.Content>
											</Select.Root>
											{#if getFieldError(`items.${index}.product_sku_id`)}
												<p class="text-sm text-destructive">
													{getFieldError(`items.${index}.product_sku_id`)}
												</p>
											{/if}
										</div>

										<div class="space-y-1 rounded-md bg-background p-3">
											<p class="text-xs text-muted-foreground">
												{t('inventory.stockAdjustments.currentStock')}
											</p>
											<p class="text-sm font-medium">
												{formatNumber(item.stock_before)}
												{item.unit}
											</p>
										</div>

										<div class="space-y-2">
											<Label
												for={`stock-adjustment-item-stock-after-${item.client_id}`}
												class="whitespace-nowrap"
											>
												{t('inventory.stockAdjustments.stockAfter')}
											</Label>
											<Input
												id={`stock-adjustment-item-stock-after-${item.client_id}`}
												type="number"
												min="0"
												step="0.001"
												disabled={isSubmitting}
												bind:value={item.stock_after}
											/>
											<p class={cn('text-xs', getDeltaClass(getItemDelta(item)))}>
												{t('inventory.stockAdjustments.detail.colDelta')}
												{formatSignedNumber(getItemDelta(item))}
											</p>
											{#if getFieldError(`items.${index}.stock_after`)}
												<p class="text-sm text-destructive">
													{getFieldError(`items.${index}.stock_after`)}
												</p>
											{/if}
										</div>

										<div class="space-y-2 md:col-span-2 lg:col-span-4">
											<Label
												for={`stock-adjustment-item-notes-${item.client_id}`}
												class="whitespace-nowrap"
											>
												{t('inventory.stockAdjustments.itemNotes')}
											</Label>
											<Textarea
												id={`stock-adjustment-item-notes-${item.client_id}`}
												placeholder={t('common.optionalNotes')}
												class="min-h-9 resize-none"
												disabled={isSubmitting}
												bind:value={item.notes}
											/>
											{#if getFieldError(`items.${index}.notes`)}
												<p class="text-sm text-destructive">
													{getFieldError(`items.${index}.notes`)}
												</p>
											{/if}
										</div>
									</div>
								</div>
							{/each}
						</div>
					</div>

					<div class="flex justify-end border-t pt-4">
						<div class="text-right">
							<p class="text-sm text-muted-foreground">
								{t('inventory.stockAdjustments.totalStockDelta')}
							</p>
							<p class={cn('text-lg font-semibold', getDeltaClass(totalDelta))}>
								{formatSignedNumber(totalDelta)}
							</p>
						</div>
					</div>

					<div class="grid gap-3">
						{#if optionsErrorMessage}
							<ErrorAlert
								title={t('inventory.stockAdjustments.errorLoadOptions')}
								message={optionsErrorMessage}
							/>
						{/if}

						{#if errorMessage}
							<ErrorAlert
								title={t('inventory.stockAdjustments.errorCreate')}
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
				<Button type="submit" disabled={isSubmitting || isLoadingOptions}>
					{#if isSubmitting}
						<Spinner />
					{/if}
					{isSubmitting
						? t('inventory.stockAdjustments.creating')
						: t('inventory.stockAdjustments.createBtn')}
				</Button>
			</Drawer.Footer>
		</form>
	</Drawer.Content>
</Drawer.Root>
