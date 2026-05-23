<script lang="ts">
	import CalendarIcon from '@lucide/svelte/icons/calendar';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import TrashIcon from '@lucide/svelte/icons/trash';
	import { parseDate, type DateValue } from '@internationalized/date';
	import { createPurchaseOrder, type PurchaseOrder } from '$lib/api/purchase-orders.js';
	import { listProductSkus, type ProductSku } from '$lib/api/product-skus.js';
	import { listSuppliers, type Supplier } from '$lib/api/suppliers.js';
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
	import { formatDate, formatIdr } from '$lib/formatters.js';
	import { cn } from '$lib/utils.js';
	import { toast } from 'svelte-sonner';
	import { z } from 'zod';

	const normalizeStringInput = (value: unknown) => {
		if (value === null || value === undefined) return '';
		return String(value);
	};

	type PurchaseOrderFormItem = {
		client_id: string;
		product_sku_id: string;
		quantity_ordered: string;
		unit_cost: string;
	};

	type PurchaseOrderForm = {
		supplier_id: string;
		order_date: string;
		expected_date: string;
		notes: string;
		items: PurchaseOrderFormItem[];
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

	const purchaseOrderItemSchema = z.object({
		client_id: z.string(),
		product_sku_id: requiredIdSchema('Product SKU'),
		quantity_ordered: positiveNumberSchema('Quantity'),
		unit_cost: nonNegativeNumberSchema('Unit cost')
	});

	const createPurchaseOrderSchema = z
		.object({
			supplier_id: requiredIdSchema('Supplier'),
			order_date: z.string().trim().min(1, 'Order date is required.'),
			expected_date: z.string().trim(),
			notes: z.string().trim().max(500, 'Notes must be at most 500 characters.'),
			items: z.array(purchaseOrderItemSchema).min(1, 'At least one line item is required.')
		})
		.superRefine((data, context) => {
			if (data.expected_date && data.expected_date < data.order_date) {
				context.addIssue({
					code: 'custom',
					path: ['expected_date'],
					message: 'Expected date must be greater than or equal to order date.'
				});
			}

			const selectedSkuIds = data.items.map((item) => item.product_sku_id).filter(Boolean);
			if (new Set(selectedSkuIds).size !== selectedSkuIds.length) {
				context.addIssue({
					code: 'custom',
					path: ['items'],
					message: 'Each line item must use a unique product SKU.'
				});
			}
		});

	let {
		open = $bindable(false),
		onCreated
	}: {
		open: boolean;
		onCreated?: (purchaseOrder: PurchaseOrder) => void | Promise<void>;
	} = $props();

	let suppliers = $state<Supplier[]>([]);
	let productSkus = $state<ProductSku[]>([]);
	let lineItemSequence = 0;
	const initialOrderDate = getTodayDate();
	let orderDateValue = $state<DateValue | undefined>(dateStringToDateValue(initialOrderDate));
	let expectedDateValue = $state<DateValue | undefined>();
	let isOrderDateOpen = $state(false);
	let isExpectedDateOpen = $state(false);
	let formData = $state<PurchaseOrderForm>(getDefaultFormData(initialOrderDate));
	let fieldErrors = $state<Record<string, string>>({});
	let isLoadingOptions = $state(false);
	let isSubmitting = $state(false);
	let errorMessage = $state<string | null>(null);
	let optionsErrorMessage = $state<string | null>(null);
	let latestOptionsRequestId = 0;

	const subtotal = $derived(formData.items.reduce((total, item) => total + getLineTotal(item), 0));

	$effect(() => {
		if (!open) return;
		resetFormData();
		fieldErrors = {};
		errorMessage = null;
		optionsErrorMessage = null;
		void loadOptions();
	});

	$effect(() => {
		const orderDate = dateValueToDateString(orderDateValue);
		if (formData.order_date !== orderDate) {
			formData.order_date = orderDate;
		}
		if (orderDateValue) {
			isOrderDateOpen = false;
		}
	});

	$effect(() => {
		const expectedDate = dateValueToDateString(expectedDateValue);
		if (formData.expected_date !== expectedDate) {
			formData.expected_date = expectedDate;
		}
		if (expectedDateValue) {
			isExpectedDateOpen = false;
		}
	});

	async function loadOptions() {
		const requestId = ++latestOptionsRequestId;
		isLoadingOptions = true;
		optionsErrorMessage = null;

		try {
			const [supplierResult, productSkuResult] = await Promise.all([
				listSuppliers({ limit: 100 }),
				listProductSkus({ limit: 100 })
			]);
			if (requestId !== latestOptionsRequestId) return;
			suppliers = supplierResult.suppliers;
			productSkus = productSkuResult.productSkus;
		} catch (error) {
			if (requestId !== latestOptionsRequestId) return;
			suppliers = [];
			productSkus = [];
			optionsErrorMessage = getErrorMessage(error, 'Failed to load purchase order options.');
		} finally {
			if (requestId === latestOptionsRequestId) {
				isLoadingOptions = false;
			}
		}
	}

	async function handleSubmit(event: SubmitEvent) {
		event.preventDefault();
		const result = createPurchaseOrderSchema.safeParse(formData);

		if (!result.success) {
			fieldErrors = getValidationErrors(result.error);
			return;
		}

		isSubmitting = true;
		errorMessage = null;
		fieldErrors = {};

		try {
			const createdPurchaseOrder = await createPurchaseOrder({
				supplier_id: Number(result.data.supplier_id),
				order_date: result.data.order_date,
				expected_date: emptyToNull(result.data.expected_date),
				notes: emptyToNull(result.data.notes),
				items: result.data.items.map((item) => ({
					product_sku_id: Number(item.product_sku_id),
					quantity_ordered: Number(item.quantity_ordered),
					unit_cost: Number(item.unit_cost)
				}))
			});
			await onCreated?.(createdPurchaseOrder);
			toast.success('Purchase order created.', {
				description: `${createdPurchaseOrder.po_no} was created as draft.`
			});
			open = false;
		} catch (error) {
			errorMessage = getErrorMessage(error, 'Failed to create purchase order.');
		} finally {
			isSubmitting = false;
		}
	}

	function addLineItem() {
		formData.items = [...formData.items, createDefaultItem()];
	}

	function removeLineItem(clientId: string) {
		if (formData.items.length === 1) return;
		formData.items = formData.items.filter((item) => item.client_id !== clientId);
	}

	function handleProductSkuChange(item: PurchaseOrderFormItem, value: string) {
		item.product_sku_id = value;
		const selectedProductSku = getProductSkuById(value);
		if (!selectedProductSku) return;

		if (!item.unit_cost || Number(item.unit_cost) === 0) {
			item.unit_cost = String(selectedProductSku.purchase_price);
		}
	}

	function getProductSkuById(value: string) {
		const id = Number(value);
		if (!Number.isFinite(id)) return null;
		return productSkus.find((productSku) => productSku.id === id) ?? null;
	}

	function getProductSkuLabel(productSku: ProductSku) {
		return `${productSku.product_name} - ${productSku.sku}`;
	}

	function getLineMeta(item: PurchaseOrderFormItem) {
		const productSku = getProductSkuById(item.product_sku_id);
		if (!productSku) return 'Select a product SKU.';
		return `${productSku.unit_name} (${productSku.unit_code}) - stock ${productSku.stock}`;
	}

	function getLineTotal(item: PurchaseOrderFormItem) {
		return toNumber(item.quantity_ordered) * toNumber(item.unit_cost);
	}

	function toNumber(value: string) {
		const number = Number(value);
		return Number.isFinite(number) ? number : 0;
	}

	function resetFormData() {
		const orderDate = getTodayDate();
		orderDateValue = dateStringToDateValue(orderDate);
		expectedDateValue = undefined;
		isOrderDateOpen = false;
		isExpectedDateOpen = false;
		formData = getDefaultFormData(orderDate);
	}

	function getDefaultFormData(orderDate = getTodayDate()): PurchaseOrderForm {
		return {
			supplier_id: '',
			order_date: orderDate,
			expected_date: '',
			notes: '',
			items: [createDefaultItem()]
		};
	}

	function createDefaultItem(): PurchaseOrderFormItem {
		lineItemSequence += 1;
		return {
			client_id: `purchase-order-item-${lineItemSequence}`,
			product_sku_id: '',
			quantity_ordered: '1',
			unit_cost: '0'
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

	function clearExpectedDate() {
		expectedDateValue = undefined;
		isExpectedDateOpen = false;
	}

	function getValidationErrors(error: z.ZodError<PurchaseOrderForm>) {
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
				<Drawer.Title>Add Purchase Order</Drawer.Title>
				<Drawer.Description>
					Create a draft purchase order with supplier and product SKU line items.
				</Drawer.Description>
			</Drawer.Header>

			<div class="flex-1 overflow-y-auto px-6 py-5">
				<div class="grid gap-5">
					<div class="grid grid-cols-1 gap-x-4 gap-y-5 md:grid-cols-2">
						<div class="space-y-2">
							<Label for="purchase-order-supplier" class="whitespace-nowrap">Supplier</Label>
							<Select.Root
								allowDeselect={false}
								type="single"
								value={formData.supplier_id}
								disabled={isSubmitting || isLoadingOptions}
								onValueChange={(value) => {
									if (typeof value === 'string') {
										formData.supplier_id = value;
									}
								}}
							>
								<Select.Trigger id="purchase-order-supplier" class="w-full">
									<Select.Value placeholder="Select supplier" />
								</Select.Trigger>
								<Select.Content>
									{#each suppliers as supplier (supplier.id)}
										<Select.Item value={String(supplier.id)} label={supplier.name}>
											{supplier.name}
										</Select.Item>
									{/each}
								</Select.Content>
							</Select.Root>
							{#if getFieldError('supplier_id')}
								<p class="text-sm text-destructive">{getFieldError('supplier_id')}</p>
							{/if}
						</div>

						<div class="space-y-2">
							<Label for="purchase-order-order-date" class="whitespace-nowrap">Order Date</Label>
							<Popover.Root bind:open={isOrderDateOpen}>
								<Popover.Trigger
									id="purchase-order-order-date"
									class={cn(
										buttonVariants({ variant: 'outline' }),
										'h-9 w-full justify-start gap-2 px-3 text-left font-normal',
										!formData.order_date && 'text-muted-foreground'
									)}
									disabled={isSubmitting}
								>
									<CalendarIcon class="size-4" />
									<span class="truncate">
										{formData.order_date ? formatDate(formData.order_date) : 'Select order date'}
									</span>
								</Popover.Trigger>
								<Popover.Content align="start" class="w-auto p-0">
									<Calendar
										type="single"
										bind:value={orderDateValue}
										class="rounded-md border-0 shadow-none"
									/>
								</Popover.Content>
							</Popover.Root>
							{#if getFieldError('order_date')}
								<p class="text-sm text-destructive">{getFieldError('order_date')}</p>
							{/if}
						</div>

						<div class="space-y-2">
							<Label for="purchase-order-expected-date" class="whitespace-nowrap">
								Expected Date
							</Label>
							<Popover.Root bind:open={isExpectedDateOpen}>
								<Popover.Trigger
									id="purchase-order-expected-date"
									class={cn(
										buttonVariants({ variant: 'outline' }),
										'h-9 w-full justify-start gap-2 px-3 text-left font-normal',
										!formData.expected_date && 'text-muted-foreground'
									)}
									disabled={isSubmitting}
								>
									<CalendarIcon class="size-4" />
									<span class="truncate">
										{formData.expected_date
											? formatDate(formData.expected_date)
											: 'Select expected date'}
									</span>
								</Popover.Trigger>
								<Popover.Content align="start" class="w-auto p-0">
									<Calendar
										type="single"
										bind:value={expectedDateValue}
										class="rounded-md border-0 shadow-none"
									/>
									{#if formData.expected_date}
										<div class="flex justify-end border-t bg-popover p-2">
											<Button type="button" variant="ghost" size="sm" onclick={clearExpectedDate}>
												Clear
											</Button>
										</div>
									{/if}
								</Popover.Content>
							</Popover.Root>
							{#if getFieldError('expected_date')}
								<p class="text-sm text-destructive">{getFieldError('expected_date')}</p>
							{/if}
						</div>

						<div class="space-y-2">
							<Label for="purchase-order-notes" class="whitespace-nowrap">Notes</Label>
							<Textarea
								id="purchase-order-notes"
								placeholder="Initial stock order"
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
						<div class="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
							<div>
								<h3 class="text-sm font-medium">Line Items</h3>
								<p class="text-sm text-muted-foreground">
									Select product SKUs, quantities, and unit costs for this order.
								</p>
							</div>
							<Button
								type="button"
								variant="outline"
								class="w-full sm:w-auto"
								disabled={isSubmitting}
								onclick={addLineItem}
							>
								<PlusIcon />
								Add item
							</Button>
						</div>

						{#if getFieldError('items')}
							<ErrorAlert title="Invalid line items." message={getFieldError('items') ?? ''} />
						{/if}

						<div class="grid gap-3">
							{#each formData.items as item, index (item.client_id)}
								<div class="grid gap-4 rounded-md border bg-muted/10 p-4">
									<div class="flex items-center justify-between gap-3">
										<p class="text-sm font-medium">Item {index + 1}</p>
										<Button
											type="button"
											variant="ghost"
											size="icon-sm"
											class="size-8 text-destructive hover:bg-destructive/10 hover:text-destructive"
											disabled={isSubmitting || formData.items.length === 1}
											onclick={() => removeLineItem(item.client_id)}
											title="Remove item"
										>
											<TrashIcon />
											<span class="sr-only">Remove item</span>
										</Button>
									</div>

									<div class="grid grid-cols-1 gap-x-4 gap-y-5 md:grid-cols-2 lg:grid-cols-4">
										<div class="space-y-2 md:col-span-2">
											<Label
												for={`purchase-order-item-sku-${item.client_id}`}
												class="whitespace-nowrap"
											>
												Product SKU
											</Label>
											<Select.Root
												allowDeselect={false}
												type="single"
												value={item.product_sku_id}
												disabled={isSubmitting || isLoadingOptions}
												onValueChange={(value) => {
													if (typeof value === 'string') {
														handleProductSkuChange(item, value);
													}
												}}
											>
												<Select.Trigger
													id={`purchase-order-item-sku-${item.client_id}`}
													class="w-full"
												>
													<Select.Value placeholder="Select product SKU" />
												</Select.Trigger>
												<Select.Content>
													{#each productSkus as productSku (productSku.id)}
														<Select.Item
															value={String(productSku.id)}
															label={getProductSkuLabel(productSku)}
														>
															{getProductSkuLabel(productSku)}
														</Select.Item>
													{/each}
												</Select.Content>
											</Select.Root>
											<p class="text-xs text-muted-foreground">{getLineMeta(item)}</p>
											{#if getFieldError(`items.${index}.product_sku_id`)}
												<p class="text-sm text-destructive">
													{getFieldError(`items.${index}.product_sku_id`)}
												</p>
											{/if}
										</div>

										<div class="space-y-2">
											<Label
												for={`purchase-order-item-quantity-${item.client_id}`}
												class="whitespace-nowrap"
											>
												Quantity
											</Label>
											<Input
												id={`purchase-order-item-quantity-${item.client_id}`}
												type="number"
												min="0.001"
												step="0.001"
												disabled={isSubmitting}
												bind:value={item.quantity_ordered}
											/>
											{#if getFieldError(`items.${index}.quantity_ordered`)}
												<p class="text-sm text-destructive">
													{getFieldError(`items.${index}.quantity_ordered`)}
												</p>
											{/if}
										</div>

										<div class="space-y-2">
											<Label
												for={`purchase-order-item-unit-cost-${item.client_id}`}
												class="whitespace-nowrap"
											>
												Unit Cost
											</Label>
											<Input
												id={`purchase-order-item-unit-cost-${item.client_id}`}
												type="number"
												min="0"
												step="1"
												disabled={isSubmitting}
												bind:value={item.unit_cost}
											/>
											{#if getFieldError(`items.${index}.unit_cost`)}
												<p class="text-sm text-destructive">
													{getFieldError(`items.${index}.unit_cost`)}
												</p>
											{/if}
										</div>
									</div>

									<div class="flex justify-end text-sm">
										<span class="text-muted-foreground">Line total:</span>
										<span class="ml-2 font-medium">{formatIdr(getLineTotal(item))}</span>
									</div>
								</div>
							{/each}
						</div>
					</div>

					<div class="flex justify-end border-t pt-4">
						<div class="text-right">
							<p class="text-sm text-muted-foreground">Subtotal</p>
							<p class="text-lg font-semibold">{formatIdr(subtotal)}</p>
						</div>
					</div>

					<div class="grid gap-3">
						{#if optionsErrorMessage}
							<ErrorAlert title="Failed to load options." message={optionsErrorMessage} />
						{/if}

						{#if errorMessage}
							<ErrorAlert title="Failed to create purchase order." message={errorMessage} />
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
					Cancel
				</Drawer.Close>
				<Button type="submit" disabled={isSubmitting || isLoadingOptions}>
					{#if isSubmitting}
						<Spinner />
					{/if}
					{isSubmitting ? 'Creating...' : 'Create purchase order'}
				</Button>
			</Drawer.Footer>
		</form>
	</Drawer.Content>
</Drawer.Root>
