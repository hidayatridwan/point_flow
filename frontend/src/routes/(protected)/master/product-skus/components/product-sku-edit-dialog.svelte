<script lang="ts" module>
	let editFormSequence = 0;

	function getNextEditFormId() {
		editFormSequence += 1;
		return `product-sku-edit-${editFormSequence}`;
	}
</script>

<script lang="ts">
	import { updateProductSku, type ProductSku } from '$lib/api/product-skus.js';
	import { ApiError } from '$lib/api/types.js';
	import { listUnits, type Unit } from '$lib/api/units.js';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import { Button, buttonVariants } from '$lib/components/ui/button/index.js';
	import * as Drawer from '$lib/components/ui/drawer/index.js';
	import * as Form from '$lib/components/ui/form/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import * as Select from '$lib/components/ui/select/index.js';
	import { Spinner } from '$lib/components/ui/spinner/index.js';
	import { cn } from '$lib/utils.js';
	import { toast } from 'svelte-sonner';
	import { defaults, superForm } from 'sveltekit-superforms';
	import { zod4, zod4Client } from 'sveltekit-superforms/adapters';
	import { z } from 'zod';
	import { t } from '$lib/i18n/locale.svelte.js';

	const normalizeStringInput = (value: unknown) => {
		if (value === null || value === undefined) return '';
		return String(value);
	};

	const positiveIdSchema = (label: string) =>
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

	const positiveNumberSchema = (label: string) =>
		nonNegativeNumberSchema(label).refine((value) => Number(value) > 0, {
			message: `${label} must be greater than zero.`
		});

	const editProductSkuSchema = z.object({
		sku: z.string().trim().min(1, 'SKU is required.').max(64, 'SKU must be at most 64 characters.'),
		barcode: z.string().trim().max(80, 'Barcode must be at most 80 characters.'),
		unit_id: positiveIdSchema('Unit'),
		contents_qty: positiveNumberSchema('Contents quantity'),
		purchase_price: nonNegativeNumberSchema('Purchase price'),
		selling_price: nonNegativeNumberSchema('Selling price'),
		stock: nonNegativeNumberSchema('Stock'),
		minimum_stock: nonNegativeNumberSchema('Minimum stock'),
		is_active: z.enum(['true', 'false'])
	});

	type EditProductSkuForm = z.infer<typeof editProductSkuSchema>;

	let {
		productSku,
		open = $bindable(false),
		onUpdated
	}: {
		productSku: ProductSku;
		open: boolean;
		onUpdated?: (productSku: ProductSku) => void;
	} = $props();

	let units = $state<Unit[]>([]);
	let isLoadingUnits = $state(false);
	let isSubmitting = $state(false);
	let errorMessage = $state<string | null>(null);
	let unitsErrorMessage = $state<string | null>(null);
	let latestUnitsRequestId = 0;

	const defaultFormData: EditProductSkuForm = {
		sku: '',
		barcode: '',
		unit_id: '',
		contents_qty: '1',
		purchase_price: '0',
		selling_price: '0',
		stock: '0',
		minimum_stock: '0',
		is_active: 'true'
	};
	const formId = getNextEditFormId();
	const productNameInputId = `${formId}-product-name`;

	const form = superForm(defaults(defaultFormData, zod4(editProductSkuSchema)), {
		id: formId,
		SPA: true,
		validationMethod: 'submit-only',
		validators: zod4Client(editProductSkuSchema)
	});
	const { form: formData, validateForm } = form;

	$effect(() => {
		if (!open) return;
		errorMessage = null;
		unitsErrorMessage = null;
		$formData = getInitialFormData(productSku);
		void loadUnits();
	});

	async function loadUnits() {
		const requestId = ++latestUnitsRequestId;
		isLoadingUnits = true;
		unitsErrorMessage = null;

		try {
			const result = await listUnits({ limit: 100 });
			if (requestId !== latestUnitsRequestId) return;
			units = result.units;
		} catch (error) {
			if (requestId !== latestUnitsRequestId) return;
			units = [];
			unitsErrorMessage = getErrorMessage(error, t('master.productSkus.errorLoadUnits'));
		} finally {
			if (requestId === latestUnitsRequestId) {
				isLoadingUnits = false;
			}
		}
	}

	async function handleSubmit(event: SubmitEvent) {
		event.preventDefault();
		const result = await validateForm({ update: true, focusOnError: true });

		if (!result.valid) return;

		isSubmitting = true;
		errorMessage = null;

		try {
			const updatedProductSku = await updateProductSku(productSku.id, {
				sku: result.data.sku,
				barcode: emptyToNull(result.data.barcode),
				unit_id: Number(result.data.unit_id),
				contents_qty: Number(result.data.contents_qty),
				purchase_price: Number(result.data.purchase_price),
				selling_price: Number(result.data.selling_price),
				stock: Number(result.data.stock),
				minimum_stock: Number(result.data.minimum_stock),
				is_active: result.data.is_active === 'true'
			});
			onUpdated?.(updatedProductSku);
			toast.success(t('master.productSkus.updatedToast'), {
				description: `Changes for ${updatedProductSku.sku} have been saved.`
			});
			open = false;
		} catch (error) {
			errorMessage = getErrorMessage(error, t('master.productSkus.errorSave'));
		} finally {
			isSubmitting = false;
		}
	}

	function getInitialFormData(productSku: ProductSku): EditProductSkuForm {
		return {
			sku: productSku.sku,
			barcode: productSku.barcode ?? '',
			unit_id: String(productSku.unit_id),
			contents_qty: String(productSku.contents_qty),
			purchase_price: String(productSku.purchase_price),
			selling_price: String(productSku.selling_price),
			stock: String(productSku.stock),
			minimum_stock: String(productSku.minimum_stock),
			is_active: String(productSku.is_active) as EditProductSkuForm['is_active']
		};
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
			class="mx-auto flex max-h-[90vh] min-h-0 w-full max-w-4xl flex-col"
			onsubmit={handleSubmit}
		>
			<Drawer.Header
				class="shrink-0 border-b px-6 py-5 text-left group-data-[vaul-drawer-direction=bottom]/drawer-content:text-left"
			>
				<Drawer.Title>Edit Product SKU</Drawer.Title>
				<Drawer.Description>Update SKU details, pricing, and stock thresholds.</Drawer.Description>
			</Drawer.Header>

			<div class="flex-1 overflow-y-auto px-6 py-5">
				<div class="grid grid-cols-1 gap-x-4 gap-y-5 sm:grid-cols-2">
					<div class="space-y-2">
						<Label for={productNameInputId} class="whitespace-nowrap">Product</Label>
						<Input
							id={productNameInputId}
							value={productSku.product_name}
							disabled
							class="w-full"
						/>
					</div>

					<Form.Field {form} name="unit_id" class="space-y-2">
						<Form.Control>
							{#snippet children({ props })}
								<Form.Label class="whitespace-nowrap">Unit</Form.Label>
								<Select.Root
									allowDeselect={false}
									type="single"
									value={$formData.unit_id}
									disabled={isSubmitting || isLoadingUnits}
									onValueChange={(value) => {
										if (typeof value === 'string') {
											$formData.unit_id = value;
										}
									}}
								>
									<Select.Trigger {...props} class="w-full">
										<Select.Value placeholder={t('master.productSkus.unit')} />
									</Select.Trigger>
									<Select.Content>
										{#each units as unit (unit.id)}
											<Select.Item value={String(unit.id)} label={`${unit.name} (${unit.code})`}>
												{unit.name} ({unit.code})
											</Select.Item>
										{/each}
									</Select.Content>
								</Select.Root>
							{/snippet}
						</Form.Control>
						<Form.FieldErrors />
					</Form.Field>

					<Form.Field {form} name="sku" class="space-y-2">
						<Form.Control>
							{#snippet children({ props })}
								<Form.Label class="whitespace-nowrap">SKU</Form.Label>
								<Input
									{...props}
									placeholder="WTR-001-PCS"
									disabled={isSubmitting}
									bind:value={$formData.sku}
								/>
							{/snippet}
						</Form.Control>
						<Form.FieldErrors />
					</Form.Field>

					<Form.Field {form} name="barcode" class="space-y-2">
						<Form.Control>
							{#snippet children({ props })}
								<Form.Label class="whitespace-nowrap">Barcode</Form.Label>
								<Input
									{...props}
									placeholder="8990000000011"
									disabled={isSubmitting}
									bind:value={$formData.barcode}
								/>
							{/snippet}
						</Form.Control>
						<Form.FieldErrors />
					</Form.Field>

					<Form.Field {form} name="contents_qty" class="space-y-2">
						<Form.Control>
							{#snippet children({ props })}
								<Form.Label class="whitespace-nowrap">Contents Qty</Form.Label>
								<Input
									{...props}
									type="number"
									min="0.001"
									step="0.001"
									disabled={isSubmitting}
									bind:value={$formData.contents_qty}
								/>
							{/snippet}
						</Form.Control>
						<Form.FieldErrors />
					</Form.Field>

					<Form.Field {form} name="stock" class="space-y-2">
						<Form.Control>
							{#snippet children({ props })}
								<Form.Label class="whitespace-nowrap">Stock</Form.Label>
								<Input
									{...props}
									type="number"
									min="0"
									step="0.001"
									disabled={isSubmitting}
									bind:value={$formData.stock}
								/>
							{/snippet}
						</Form.Control>
						<Form.FieldErrors />
					</Form.Field>

					<Form.Field {form} name="minimum_stock" class="space-y-2">
						<Form.Control>
							{#snippet children({ props })}
								<Form.Label class="whitespace-nowrap">Minimum Stock</Form.Label>
								<Input
									{...props}
									type="number"
									min="0"
									step="0.001"
									disabled={isSubmitting}
									bind:value={$formData.minimum_stock}
								/>
							{/snippet}
						</Form.Control>
						<Form.FieldErrors />
					</Form.Field>

					<Form.Field {form} name="is_active" class="space-y-2">
						<Form.Control>
							{#snippet children({ props })}
								<Form.Label class="whitespace-nowrap">Status</Form.Label>
								<Select.Root
									allowDeselect={false}
									type="single"
									value={$formData.is_active}
									disabled={isSubmitting}
									onValueChange={(value) => {
										if (value === 'true' || value === 'false') {
											$formData.is_active = value;
										}
									}}
								>
									<Select.Trigger {...props} class="w-full">
										<Select.Value placeholder={t('common.selectStatus')} />
									</Select.Trigger>
									<Select.Content>
										<Select.Item value="true" label="Active">{t('status.active')}</Select.Item>
										<Select.Item value="false" label="Inactive">{t('status.inactive')}</Select.Item>
									</Select.Content>
								</Select.Root>
							{/snippet}
						</Form.Control>
						<Form.FieldErrors />
					</Form.Field>

					<Form.Field {form} name="purchase_price" class="space-y-2">
						<Form.Control>
							{#snippet children({ props })}
								<Form.Label class="whitespace-nowrap">Purchase Price</Form.Label>
								<Input
									{...props}
									type="number"
									min="0"
									step="1"
									disabled={isSubmitting}
									bind:value={$formData.purchase_price}
								/>
							{/snippet}
						</Form.Control>
						<Form.FieldErrors />
					</Form.Field>

					<Form.Field {form} name="selling_price" class="space-y-2">
						<Form.Control>
							{#snippet children({ props })}
								<Form.Label class="whitespace-nowrap">Selling Price</Form.Label>
								<Input
									{...props}
									type="number"
									min="0"
									step="1"
									disabled={isSubmitting}
									bind:value={$formData.selling_price}
								/>
							{/snippet}
						</Form.Control>
						<Form.FieldErrors />
					</Form.Field>
				</div>

				<div class="mt-5 grid gap-3">
					{#if unitsErrorMessage}
						<ErrorAlert title={t('master.productSkus.errorLoadUnits')} message={unitsErrorMessage} />
					{/if}

					{#if errorMessage}
						<ErrorAlert title={t('common.saveFailed')} message={errorMessage} />
					{/if}
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
				<Button type="submit" disabled={isSubmitting || isLoadingUnits}>
					{#if isSubmitting}
						<Spinner />
					{/if}
					{isSubmitting ? t('common.saving') : t('common.save')}
				</Button>
			</Drawer.Footer>
		</form>
	</Drawer.Content>
</Drawer.Root>
