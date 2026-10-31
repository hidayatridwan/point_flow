<script lang="ts">
	import CalendarIcon from '@lucide/svelte/icons/calendar';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import TrashIcon from '@lucide/svelte/icons/trash';
	import { parseDate, type DateValue } from '@internationalized/date';
	import { listProductSkus, type ProductSku } from '$lib/api/product-skus.js';
	import { createStockConversion, type StockConversion } from '$lib/api/stock-conversions.js';
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

	type StockConversionFormItem = {
		client_id: string;
		from_product_sku_id: string;
		from_product_id: number;
		from_product_name: string;
		from_product_sku: string;
		from_unit: string;
		from_contents_qty: number;
		from_stock_before: number;
		from_quantity: string;
		to_product_sku_id: string;
		to_product_id: number;
		to_product_name: string;
		to_product_sku: string;
		to_unit: string;
		to_contents_qty: number;
		to_stock_before: number;
		to_quantity: string;
		notes: string;
	};

	type StockConversionForm = {
		converted_date: string;
		notes: string;
		items: StockConversionFormItem[];
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

	const stockConversionItemSchema = z.object({
		client_id: z.string(),
		from_product_sku_id: requiredIdSchema('Source SKU'),
		from_product_id: z.number(),
		from_product_name: z.string(),
		from_product_sku: z.string(),
		from_unit: z.string(),
		from_contents_qty: z.number(),
		from_stock_before: z.number(),
		from_quantity: positiveNumberSchema('Source quantity'),
		to_product_sku_id: requiredIdSchema('Target SKU'),
		to_product_id: z.number(),
		to_product_name: z.string(),
		to_product_sku: z.string(),
		to_unit: z.string(),
		to_contents_qty: z.number(),
		to_stock_before: z.number(),
		to_quantity: positiveNumberSchema('Target quantity'),
		notes: z.string().trim().max(300, 'Item notes must be at most 300 characters.')
	});

	const createStockConversionSchema = z
		.object({
			converted_date: z.string().trim().min(1, 'Converted date is required.'),
			notes: z.string().trim().max(500, 'Notes must be at most 500 characters.'),
			items: z.array(stockConversionItemSchema).min(1, 'At least one line item is required.')
		})
		.superRefine((data, context) => {
			let hasDuplicateSkuAcrossLines = false;
			const selectedSkuLineIndexes: Record<string, number | undefined> = {};

			data.items.forEach((item, index) => {
				if (item.from_product_sku_id && item.from_product_sku_id === item.to_product_sku_id) {
					context.addIssue({
						code: 'custom',
						path: ['items', index, 'to_product_sku_id'],
						message: 'Target SKU must be different from source SKU.'
					});
				}

				if (
					item.from_product_id > 0 &&
					item.to_product_id > 0 &&
					item.from_product_id !== item.to_product_id
				) {
					context.addIssue({
						code: 'custom',
						path: ['items', index, 'to_product_sku_id'],
						message: 'Target SKU must belong to the same product as source SKU.'
					});
				}

				if (Number(item.from_quantity) > item.from_stock_before) {
					context.addIssue({
						code: 'custom',
						path: ['items', index, 'from_quantity'],
						message: 'Source quantity must not exceed current stock.'
					});
				}

				for (const selectedSkuId of [item.from_product_sku_id, item.to_product_sku_id]) {
					if (!selectedSkuId) continue;
					const previousLineIndex = selectedSkuLineIndexes[selectedSkuId];
					if (previousLineIndex !== undefined && previousLineIndex !== index) {
						hasDuplicateSkuAcrossLines = true;
					}
					selectedSkuLineIndexes[selectedSkuId] = index;
				}
			});

			if (hasDuplicateSkuAcrossLines) {
				context.addIssue({
					code: 'custom',
					path: ['items'],
					message: 'Each product SKU can only appear once across conversion lines.'
				});
			}
		});

	let {
		open = $bindable(false),
		onCreated
	}: {
		open: boolean;
		onCreated?: (stockConversion: StockConversion) => void | Promise<void>;
	} = $props();

	let productSkus = $state<ProductSku[]>([]);
	let lineItemSequence = 0;
	const initialConvertedDate = getTodayDate();
	let convertedDateValue = $state<DateValue | undefined>(
		dateStringToDateValue(initialConvertedDate)
	);
	let isConvertedDateOpen = $state(false);
	let formData = $state<StockConversionForm>(getDefaultFormData(initialConvertedDate));
	let fieldErrors = $state<Record<string, string>>({});
	let isLoadingOptions = $state(false);
	let isSubmitting = $state(false);
	let errorMessage = $state<string | null>(null);
	let optionsErrorMessage = $state<string | null>(null);
	let latestOptionsRequestId = 0;

	$effect(() => {
		if (!open) return;
		resetFormData();
		fieldErrors = {};
		errorMessage = null;
		optionsErrorMessage = null;
		void loadOptions();
	});

	$effect(() => {
		const convertedDate = dateValueToDateString(convertedDateValue);
		if (formData.converted_date !== convertedDate) {
			formData.converted_date = convertedDate;
		}
		if (convertedDateValue) {
			isConvertedDateOpen = false;
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
		const result = createStockConversionSchema.safeParse(formData);

		if (!result.success) {
			fieldErrors = getValidationErrors(result.error);
			return;
		}

		isSubmitting = true;
		errorMessage = null;
		fieldErrors = {};

		try {
			const createdStockConversion = await createStockConversion({
				converted_at: dateToUtcDateTime(result.data.converted_date),
				notes: emptyToNull(result.data.notes),
				items: result.data.items.map((item) => ({
					from_product_sku_id: Number(item.from_product_sku_id),
					from_quantity: Number(item.from_quantity),
					to_product_sku_id: Number(item.to_product_sku_id),
					to_quantity: Number(item.to_quantity),
					notes: emptyToNull(item.notes)
				}))
			});
			await onCreated?.(createdStockConversion);
			toast.success(t('inventory.stockConversions.createdToast'), {
				description: `${createdStockConversion.conversion_no} was created as draft.`
			});
			open = false;
		} catch (error) {
			errorMessage = getErrorMessage(error, t('inventory.stockConversions.errorCreate'));
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

	function updateLineItemFromProductSku(clientId: string, value: string) {
		const productSku = productSkus.find((item) => String(item.id) === value);
		formData.items = formData.items.map((item) => {
			if (item.client_id !== clientId) return item;
			const nextItem = productSku
				? {
						...item,
						from_product_sku_id: String(productSku.id),
						from_product_id: productSku.product_id,
						from_product_name: productSku.product_name,
						from_product_sku: productSku.sku,
						from_unit: productSku.unit_code,
						from_contents_qty: productSku.contents_qty,
						from_stock_before: productSku.stock,
						from_quantity: item.from_quantity,
						notes: item.notes
					}
				: {
						...item,
						from_product_sku_id: '',
						from_product_id: 0,
						from_product_name: '',
						from_product_sku: '',
						from_unit: '',
						from_contents_qty: 0,
						from_stock_before: 0,
						from_quantity: item.from_quantity,
						notes: item.notes
					};

			if (productSku && item.to_product_id > 0 && item.to_product_id !== productSku.product_id) {
				return clearTargetSku(nextItem);
			}

			return withCalculatedTargetQuantity(nextItem);
		});
	}

	function updateLineItemFromQuantity(clientId: string, value: string) {
		formData.items = formData.items.map((item) => {
			if (item.client_id !== clientId) return item;
			return withCalculatedTargetQuantity({
				...item,
				from_quantity: value
			});
		});
	}

	function updateLineItemToProductSku(clientId: string, value: string) {
		const productSku = productSkus.find((item) => String(item.id) === value);
		formData.items = formData.items.map((item) => {
			if (item.client_id !== clientId) return item;
			if (!productSku) return clearTargetSku(item);

			return withCalculatedTargetQuantity({
				...item,
				to_product_sku_id: String(productSku.id),
				to_product_id: productSku.product_id,
				to_product_name: productSku.product_name,
				to_product_sku: productSku.sku,
				to_unit: productSku.unit_code,
				to_contents_qty: productSku.contents_qty,
				to_stock_before: productSku.stock,
				notes: item.notes
			});
		});
	}

	function clearTargetSku(item: StockConversionFormItem): StockConversionFormItem {
		return {
			...item,
			to_product_sku_id: '',
			to_product_id: 0,
			to_product_name: '',
			to_product_sku: '',
			to_unit: '',
			to_contents_qty: 0,
			to_stock_before: 0,
			to_quantity: ''
		};
	}

	function withCalculatedTargetQuantity(item: StockConversionFormItem): StockConversionFormItem {
		return {
			...item,
			to_quantity: calculateTargetQuantity(item)
		};
	}

	function calculateTargetQuantity(item: StockConversionFormItem) {
		const fromQuantity = Number(item.from_quantity);
		if (
			!Number.isFinite(fromQuantity) ||
			fromQuantity <= 0 ||
			item.from_contents_qty <= 0 ||
			item.to_contents_qty <= 0
		) {
			return '';
		}

		return formatQuantityInput((fromQuantity * item.from_contents_qty) / item.to_contents_qty);
	}

	function formatQuantityInput(value: number) {
		const roundedValue = Math.round((value + Number.EPSILON) * 1000) / 1000;
		return String(roundedValue);
	}

	function createEmptyItem(): StockConversionFormItem {
		lineItemSequence += 1;
		return {
			client_id: `stock-conversion-item-${lineItemSequence}`,
			from_product_sku_id: '',
			from_product_id: 0,
			from_product_name: '',
			from_product_sku: '',
			from_unit: '',
			from_contents_qty: 0,
			from_stock_before: 0,
			from_quantity: '',
			to_product_sku_id: '',
			to_product_id: 0,
			to_product_name: '',
			to_product_sku: '',
			to_unit: '',
			to_contents_qty: 0,
			to_stock_before: 0,
			to_quantity: '',
			notes: ''
		};
	}

	function getProductSkuLabel(productSku: ProductSku) {
		return `${productSku.product_name} - ${productSku.sku} (${productSku.unit_code})`;
	}

	function getSourceProductSkus(item: StockConversionFormItem) {
		return productSkus.filter((productSku) => {
			if (String(productSku.id) === item.to_product_sku_id) return false;
			return isProductSkuAvailableForItem(productSku.id, item.client_id);
		});
	}

	function getTargetProductSkus(item: StockConversionFormItem) {
		return productSkus.filter((productSku) => {
			if (String(productSku.id) === item.from_product_sku_id) return false;
			if (item.from_product_id > 0 && productSku.product_id !== item.from_product_id) return false;
			return isProductSkuAvailableForItem(productSku.id, item.client_id);
		});
	}

	function isProductSkuAvailableForItem(productSkuId: number, clientId: string) {
		const value = String(productSkuId);
		return !formData.items.some(
			(item) =>
				item.client_id !== clientId &&
				(item.from_product_sku_id === value || item.to_product_sku_id === value)
		);
	}

	function getSourceStockAfter(item: StockConversionFormItem) {
		return item.from_stock_before - toNumber(item.from_quantity);
	}

	function getTargetStockAfter(item: StockConversionFormItem) {
		return item.to_stock_before + toNumber(item.to_quantity);
	}

	function getConversionRatioLabel(item: StockConversionFormItem) {
		if (item.from_contents_qty <= 0 || item.to_contents_qty <= 0) return '';
		const targetQuantity = item.from_contents_qty / item.to_contents_qty;
		return `1 ${item.from_unit} = ${formatNumber(targetQuantity)} ${item.to_unit}`;
	}

	function toNumber(value: string) {
		const number = Number(value);
		return Number.isFinite(number) ? number : 0;
	}

	function resetFormData() {
		const convertedDate = getTodayDate();
		convertedDateValue = dateStringToDateValue(convertedDate);
		isConvertedDateOpen = false;
		formData = getDefaultFormData(convertedDate);
	}

	function getDefaultFormData(convertedDate = getTodayDate()): StockConversionForm {
		return {
			converted_date: convertedDate,
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
			class="mx-auto flex max-h-[90vh] min-h-0 w-full max-w-6xl flex-col"
			onsubmit={handleSubmit}
		>
			<Drawer.Header
				class="shrink-0 border-b px-6 py-5 text-left group-data-[vaul-drawer-direction=bottom]/drawer-content:text-left"
			>
				<Drawer.Title>{t('inventory.stockConversions.addTitle')}</Drawer.Title>
				<Drawer.Description>{t('inventory.stockConversions.addDesc')}</Drawer.Description>
			</Drawer.Header>

			<div class="flex-1 overflow-y-auto px-6 py-5">
				<div class="grid gap-5">
					<div class="grid grid-cols-1 gap-x-4 gap-y-5 md:grid-cols-2">
						<div class="space-y-2">
							<Label for="stock-conversion-converted-date" class="whitespace-nowrap">
								{t('inventory.stockConversions.convertedDate')}
							</Label>
							<Popover.Root bind:open={isConvertedDateOpen}>
								<Popover.Trigger
									id="stock-conversion-converted-date"
									class={cn(
										buttonVariants({ variant: 'outline' }),
										'h-9 w-full justify-start gap-2 px-3 text-left font-normal',
										!formData.converted_date && 'text-muted-foreground'
									)}
									disabled={isSubmitting}
								>
									<CalendarIcon class="size-4" />
									<span class="truncate">
										{formData.converted_date
											? formatDate(formData.converted_date)
											: t('inventory.stockConversions.selectConvertedDate')}
									</span>
								</Popover.Trigger>
								<Popover.Content align="start" class="w-auto p-0">
									<Calendar
										type="single"
										bind:value={convertedDateValue}
										class="rounded-md border-0 shadow-none"
									/>
								</Popover.Content>
							</Popover.Root>
							{#if getFieldError('converted_date')}
								<p class="text-sm text-destructive">{getFieldError('converted_date')}</p>
							{/if}
						</div>

						<div class="space-y-2 md:col-span-2">
							<Label for="stock-conversion-notes" class="whitespace-nowrap"
								>{t('common.notes')}</Label
							>
							<Textarea
								id="stock-conversion-notes"
								placeholder={t('inventory.stockConversions.notesPlaceholder')}
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
									{t('inventory.stockConversions.conversionItems')}
								</h3>
								<p class="text-sm text-muted-foreground">
									{t('inventory.stockConversions.conversionItemsDesc')}
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
								title={t('inventory.stockConversions.invalidItems')}
								message={getFieldError('items') ?? ''}
							/>
						{/if}

						<div class="grid gap-3">
							{#each formData.items as item, index (item.client_id)}
								<div class="grid gap-4 rounded-md border bg-muted/10 p-4">
									<div class="flex items-start justify-between gap-3">
										<div>
											<p class="text-sm font-medium">
												{t('common.itemNo').replace('{index}', String(index + 1))}
											</p>
											<p class="text-xs text-muted-foreground">
												{item.from_product_name ||
													item.to_product_name ||
													t('inventory.stockConversions.selectProductSkus')}
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

									<div class="grid grid-cols-1 gap-x-4 gap-y-5 lg:grid-cols-6">
										<div class="space-y-2 lg:col-span-2">
											<Label
												for={`stock-conversion-item-from-sku-${item.client_id}`}
												class="whitespace-nowrap"
											>
												{t('inventory.stockConversions.sourceSku')}
											</Label>
											<Select.Root
												allowDeselect={false}
												type="single"
												value={item.from_product_sku_id}
												disabled={isSubmitting || isLoadingOptions}
												onValueChange={(value) => {
													if (typeof value === 'string') {
														updateLineItemFromProductSku(item.client_id, value);
													}
												}}
											>
												<Select.Trigger
													id={`stock-conversion-item-from-sku-${item.client_id}`}
													class="w-full"
												>
													<Select.Value
														placeholder={t('inventory.stockConversions.selectSourceSku')}
													/>
												</Select.Trigger>
												<Select.Content>
													{#each getSourceProductSkus(item) as productSku (productSku.id)}
														<Select.Item
															value={String(productSku.id)}
															label={getProductSkuLabel(productSku)}
														>
															{getProductSkuLabel(productSku)}
														</Select.Item>
													{/each}
												</Select.Content>
											</Select.Root>
											{#if getFieldError(`items.${index}.from_product_sku_id`)}
												<p class="text-sm text-destructive">
													{getFieldError(`items.${index}.from_product_sku_id`)}
												</p>
											{/if}
										</div>

										<div class="space-y-1 rounded-md bg-background p-3">
											<p class="text-xs text-muted-foreground">
												{t('inventory.stockConversions.sourceStock')}
											</p>
											<p class="text-sm font-medium">
												{formatNumber(item.from_stock_before)}
												{item.from_unit}
											</p>
											<p class="text-xs text-muted-foreground">
												{t('common.colAfter')}
												{formatNumber(getSourceStockAfter(item))}
												{item.from_unit}
											</p>
										</div>

										<div class="space-y-2">
											<Label
												for={`stock-conversion-item-from-quantity-${item.client_id}`}
												class="whitespace-nowrap"
											>
												{t('inventory.stockConversions.sourceQty')}
											</Label>
											<Input
												id={`stock-conversion-item-from-quantity-${item.client_id}`}
												type="number"
												min="0"
												step="0.001"
												disabled={isSubmitting}
												value={item.from_quantity}
												oninput={(event) =>
													updateLineItemFromQuantity(item.client_id, event.currentTarget.value)}
											/>
											{#if getFieldError(`items.${index}.from_quantity`)}
												<p class="text-sm text-destructive">
													{getFieldError(`items.${index}.from_quantity`)}
												</p>
											{/if}
										</div>

										<div class="space-y-2 lg:col-span-2">
											<Label
												for={`stock-conversion-item-to-sku-${item.client_id}`}
												class="whitespace-nowrap"
											>
												{t('inventory.stockConversions.targetSku')}
											</Label>
											<Select.Root
												allowDeselect={false}
												type="single"
												value={item.to_product_sku_id}
												disabled={isSubmitting || isLoadingOptions}
												onValueChange={(value) => {
													if (typeof value === 'string') {
														updateLineItemToProductSku(item.client_id, value);
													}
												}}
											>
												<Select.Trigger
													id={`stock-conversion-item-to-sku-${item.client_id}`}
													class="w-full"
												>
													<Select.Value
														placeholder={t('inventory.stockConversions.selectTargetSku')}
													/>
												</Select.Trigger>
												<Select.Content>
													{#each getTargetProductSkus(item) as productSku (productSku.id)}
														<Select.Item
															value={String(productSku.id)}
															label={getProductSkuLabel(productSku)}
														>
															{getProductSkuLabel(productSku)}
														</Select.Item>
													{/each}
												</Select.Content>
											</Select.Root>
											{#if getFieldError(`items.${index}.to_product_sku_id`)}
												<p class="text-sm text-destructive">
													{getFieldError(`items.${index}.to_product_sku_id`)}
												</p>
											{/if}
										</div>

										<div class="space-y-1 rounded-md bg-background p-3">
											<p class="text-xs text-muted-foreground">
												{t('inventory.stockConversions.targetStock')}
											</p>
											<p class="text-sm font-medium">
												{formatNumber(item.to_stock_before)}
												{item.to_unit}
											</p>
											<p class="text-xs text-muted-foreground">
												{t('common.colAfter')}
												{formatNumber(getTargetStockAfter(item))}
												{item.to_unit}
											</p>
										</div>

										<div class="space-y-2">
											<Label
												for={`stock-conversion-item-to-quantity-${item.client_id}`}
												class="whitespace-nowrap"
											>
												{t('inventory.stockConversions.targetQty')}
											</Label>
											<Input
												id={`stock-conversion-item-to-quantity-${item.client_id}`}
												type="number"
												min="0"
												step="0.001"
												readonly
												disabled={isSubmitting}
												value={item.to_quantity}
											/>
											{#if getConversionRatioLabel(item)}
												<p class="text-xs text-muted-foreground">
													{getConversionRatioLabel(item)}
												</p>
											{/if}
											{#if getFieldError(`items.${index}.to_quantity`)}
												<p class="text-sm text-destructive">
													{getFieldError(`items.${index}.to_quantity`)}
												</p>
											{/if}
										</div>

										<div class="space-y-2 lg:col-span-6">
											<Label
												for={`stock-conversion-item-notes-${item.client_id}`}
												class="whitespace-nowrap"
											>
												{t('inventory.stockConversions.itemNotes')}
											</Label>
											<Textarea
												id={`stock-conversion-item-notes-${item.client_id}`}
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

					<div class="grid gap-3">
						{#if optionsErrorMessage}
							<ErrorAlert
								title={t('inventory.stockConversions.errorLoadOptions')}
								message={optionsErrorMessage}
							/>
						{/if}

						{#if errorMessage}
							<ErrorAlert
								title={t('inventory.stockConversions.errorCreate')}
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
						? t('inventory.stockConversions.creating')
						: t('inventory.stockConversions.createBtn')}
				</Button>
			</Drawer.Footer>
		</form>
	</Drawer.Content>
</Drawer.Root>
