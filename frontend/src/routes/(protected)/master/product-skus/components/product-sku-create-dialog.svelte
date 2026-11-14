<script lang="ts">
	import { createProductSku, type ProductSku } from '$lib/api/product-skus.js';
	import { listProducts, type Product } from '$lib/api/products.js';
	import { ApiError } from '$lib/api/types.js';
	import { listUnits, type Unit } from '$lib/api/units.js';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import { Button, buttonVariants } from '$lib/components/ui/button/index.js';
	import * as Form from '$lib/components/ui/form/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import * as Drawer from '$lib/components/ui/drawer/index.js';
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

	const createProductSkuSchema = z.object({
		product_id: positiveIdSchema('Product'),
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

	type CreateProductSkuForm = z.infer<typeof createProductSkuSchema>;

	let {
		open = $bindable(false),
		onCreated
	}: {
		open: boolean;
		onCreated?: (productSku: ProductSku) => void | Promise<void>;
	} = $props();

	let products = $state<Product[]>([]);
	let units = $state<Unit[]>([]);
	let isLoadingOptions = $state(false);
	let isSubmitting = $state(false);
	let errorMessage = $state<string | null>(null);

	// Label source untuk Select.Root: tanpa ini, trigger menampilkan nilai mentah
	// ("true") sampai dropdown dibuka pertama kali (bits-ui lazy items).
	const productItems = $derived(
		products.map((product) => ({ value: String(product.id), label: product.name }))
	);
	const unitItems = $derived(
		units.map((unit) => ({ value: String(unit.id), label: `${unit.name} (${unit.code})` }))
	);
	const statusItems = $derived([
		{ value: 'true', label: t('status.active') },
		{ value: 'false', label: t('status.inactive') }
	]);
	let optionsErrorMessage = $state<string | null>(null);
	let latestOptionsRequestId = 0;

	const defaultFormData: CreateProductSkuForm = {
		product_id: '',
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

	const form = superForm(defaults(defaultFormData, zod4(createProductSkuSchema)), {
		id: 'product-sku-create',
		SPA: true,
		validationMethod: 'submit-only',
		validators: zod4Client(createProductSkuSchema)
	});
	const { form: formData, validateForm } = form;

	$effect(() => {
		if (!open) return;
		errorMessage = null;
		optionsErrorMessage = null;
		$formData = { ...defaultFormData };
		void loadOptions();
	});

	async function loadOptions() {
		const requestId = ++latestOptionsRequestId;
		isLoadingOptions = true;
		optionsErrorMessage = null;

		try {
			const [productResult, unitResult] = await Promise.all([
				listProducts({ limit: 100 }),
				listUnits({ limit: 100 })
			]);
			if (requestId !== latestOptionsRequestId) return;
			products = productResult.products;
			units = unitResult.units;
		} catch (error) {
			if (requestId !== latestOptionsRequestId) return;
			products = [];
			units = [];
			optionsErrorMessage = getErrorMessage(error, t('master.productSkus.errorLoadOptions'));
		} finally {
			if (requestId === latestOptionsRequestId) {
				isLoadingOptions = false;
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
			const createdProductSku = await createProductSku({
				product_id: Number(result.data.product_id),
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
			await onCreated?.(createdProductSku);
			toast.success(t('master.productSkus.createdToast'), {
				description: `${createdProductSku.sku} ${t('common.wasAdded')}`
			});
			open = false;
		} catch (error) {
			errorMessage = getErrorMessage(error, t('master.productSkus.errorCreate'));
		} finally {
			isSubmitting = false;
		}
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
				<Drawer.Title>{t('master.productSkus.addTitle')}</Drawer.Title>
				<Drawer.Description>
					{t('master.productSkus.addDesc')}
				</Drawer.Description>
			</Drawer.Header>

			<div class="flex-1 overflow-y-auto px-6 py-5">
				<div class="grid grid-cols-1 gap-x-4 gap-y-5 sm:grid-cols-2">
					<Form.Field {form} name="product_id" class="space-y-2">
						<Form.Control>
							{#snippet children({ props })}
								<Form.Label class="whitespace-nowrap">{t('master.productSkus.product')}</Form.Label>
								<Select.Root
									allowDeselect={false}
									type="single"
									items={productItems}
									value={$formData.product_id}
									disabled={isSubmitting || isLoadingOptions}
									onValueChange={(value) => {
										if (typeof value === 'string') {
											$formData.product_id = value;
										}
									}}
								>
									<Select.Trigger {...props} class="w-full">
										<Select.Value placeholder={t('master.productSkus.selectProduct')} />
									</Select.Trigger>
									<Select.Content>
										{#each products as product (product.id)}
											<Select.Item value={String(product.id)} label={product.name}>
												{product.name}
											</Select.Item>
										{/each}
									</Select.Content>
								</Select.Root>
							{/snippet}
						</Form.Control>
						<Form.FieldErrors />
					</Form.Field>

					<Form.Field {form} name="unit_id" class="space-y-2">
						<Form.Control>
							{#snippet children({ props })}
								<Form.Label class="whitespace-nowrap">{t('master.productSkus.unit')}</Form.Label>
								<Select.Root
									allowDeselect={false}
									type="single"
									items={unitItems}
									value={$formData.unit_id}
									disabled={isSubmitting || isLoadingOptions}
									onValueChange={(value) => {
										if (typeof value === 'string') {
											$formData.unit_id = value;
										}
									}}
								>
									<Select.Trigger {...props} class="w-full">
										<Select.Value placeholder={t('master.productSkus.selectUnit')} />
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
								<Form.Label class="whitespace-nowrap">{t('master.productSkus.sku')}</Form.Label>
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
								<Form.Label class="whitespace-nowrap">{t('master.productSkus.barcode')}</Form.Label>
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
								<Form.Label class="whitespace-nowrap"
									>{t('master.productSkus.contentsQty')}</Form.Label
								>
								<Input
									{...props}
									type="number"
									min="0.001"
									step="0.001"
									placeholder="1"
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
								<Form.Label class="whitespace-nowrap">{t('master.productSkus.stock')}</Form.Label>
								<Input
									{...props}
									type="number"
									min="0"
									step="0.001"
									placeholder="0"
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
								<Form.Label class="whitespace-nowrap"
									>{t('master.productSkus.minimumStock')}</Form.Label
								>
								<Input
									{...props}
									type="number"
									min="0"
									step="0.001"
									placeholder="0"
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
								<Form.Label class="whitespace-nowrap">{t('common.status')}</Form.Label>
								<Select.Root
									allowDeselect={false}
									type="single"
									items={statusItems}
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
										<Select.Item value="true" label={t('status.active')}
											>{t('status.active')}</Select.Item
										>
										<Select.Item value="false" label={t('status.inactive')}
											>{t('status.inactive')}</Select.Item
										>
									</Select.Content>
								</Select.Root>
							{/snippet}
						</Form.Control>
						<Form.FieldErrors />
					</Form.Field>

					<Form.Field {form} name="purchase_price" class="space-y-2">
						<Form.Control>
							{#snippet children({ props })}
								<Form.Label class="whitespace-nowrap"
									>{t('master.productSkus.purchasePrice')}</Form.Label
								>
								<Input
									{...props}
									type="number"
									min="0"
									step="1"
									placeholder="2500"
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
								<Form.Label class="whitespace-nowrap"
									>{t('master.productSkus.sellingPrice')}</Form.Label
								>
								<Input
									{...props}
									type="number"
									min="0"
									step="1"
									placeholder="4000"
									disabled={isSubmitting}
									bind:value={$formData.selling_price}
								/>
							{/snippet}
						</Form.Control>
						<Form.FieldErrors />
					</Form.Field>
				</div>

				<div class="mt-5 grid gap-3">
					{#if optionsErrorMessage}
						<ErrorAlert
							title={t('master.productSkus.errorLoadOptions')}
							message={optionsErrorMessage}
						/>
					{/if}

					{#if errorMessage}
						<ErrorAlert title={t('master.productSkus.errorCreate')} message={errorMessage} />
					{/if}
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
					{isSubmitting ? t('master.productSkus.creating') : t('master.productSkus.createBtn')}
				</Button>
			</Drawer.Footer>
		</form>
	</Drawer.Content>
</Drawer.Root>
