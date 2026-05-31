<script lang="ts">
	import { listProductCategories, type ProductCategory } from '$lib/api/product-categories.js';
	import { createProduct, type Product } from '$lib/api/products.js';
	import { ApiError } from '$lib/api/types.js';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import { Button, buttonVariants } from '$lib/components/ui/button/index.js';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import * as Form from '$lib/components/ui/form/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import * as Select from '$lib/components/ui/select/index.js';
	import { Spinner } from '$lib/components/ui/spinner/index.js';
	import { cn } from '$lib/utils.js';
	import { toast } from 'svelte-sonner';
	import { defaults, superForm } from 'sveltekit-superforms';
	import { zod4, zod4Client } from 'sveltekit-superforms/adapters';
	import { z } from 'zod';
	import { t } from '$lib/i18n/locale.svelte.js';

	const NO_CATEGORY_VALUE = 'none';

	const createProductSchema = z.object({
		name: z
			.string()
			.trim()
			.min(1, 'Name is required.')
			.max(160, 'Name must be at most 160 characters.'),
		category_id: z.string(),
		is_active: z.enum(['true', 'false'])
	});

	type CreateProductForm = z.infer<typeof createProductSchema>;

	let {
		open = $bindable(false),
		onCreated
	}: {
		open: boolean;
		onCreated?: (product: Product) => void | Promise<void>;
	} = $props();

	let categories = $state<ProductCategory[]>([]);
	let isLoadingCategories = $state(false);
	let isSubmitting = $state(false);
	let errorMessage = $state<string | null>(null);
	let categoryErrorMessage = $state<string | null>(null);
	let latestCategoryRequestId = 0;

	const defaultFormData: CreateProductForm = {
		name: '',
		category_id: NO_CATEGORY_VALUE,
		is_active: 'true'
	};

	const form = superForm(defaults(defaultFormData, zod4(createProductSchema)), {
		id: 'product-create',
		SPA: true,
		validationMethod: 'submit-only',
		validators: zod4Client(createProductSchema)
	});
	const { form: formData, validateForm } = form;

	$effect(() => {
		if (!open) return;
		errorMessage = null;
		categoryErrorMessage = null;
		$formData = { ...defaultFormData };
		void loadCategories();
	});

	async function loadCategories() {
		const requestId = ++latestCategoryRequestId;
		isLoadingCategories = true;
		categoryErrorMessage = null;

		try {
			const result = await listProductCategories({ limit: 100 });
			if (requestId !== latestCategoryRequestId) return;
			categories = result.categories;
		} catch (error) {
			if (requestId !== latestCategoryRequestId) return;
			categories = [];
			categoryErrorMessage = getErrorMessage(error, 'Failed to load product categories.');
		} finally {
			if (requestId === latestCategoryRequestId) {
				isLoadingCategories = false;
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
			const createdProduct = await createProduct({
				name: result.data.name,
				category_id: getNullableCategoryId(result.data.category_id),
				is_active: result.data.is_active === 'true'
			});
			await onCreated?.(createdProduct);
			toast.success(t('master.products.createdToast'), {
				description: `${createdProduct.name} is ready for SKU setup.`
			});
			open = false;
		} catch (error) {
			errorMessage = getErrorMessage(error, t('master.products.errorCreate'));
		} finally {
			isSubmitting = false;
		}
	}

	function getNullableCategoryId(value: string) {
		if (value === NO_CATEGORY_VALUE) return null;
		const id = Number(value);
		return Number.isFinite(id) ? id : null;
	}

	function getErrorMessage(error: unknown, fallback: string) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return fallback;
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="overflow-hidden p-0 sm:max-w-lg">
		<form class="grid gap-6" onsubmit={handleSubmit}>
			<Dialog.Header class="px-6 pt-6">
				<Dialog.Title>{t('master.products.addTitle')}</Dialog.Title>
				<Dialog.Description>
					Create a product master record before adding SKU, pricing, and stock data.
				</Dialog.Description>
			</Dialog.Header>

			<div class="grid gap-4 px-6">
				<Form.Field {form} name="name">
					<Form.Control>
						{#snippet children({ props })}
							<Form.Label>Name</Form.Label>
							<Input
								{...props}
								placeholder="Mineral Water"
								disabled={isSubmitting}
								bind:value={$formData.name}
							/>
						{/snippet}
					</Form.Control>
					<Form.FieldErrors />
				</Form.Field>

				<Form.Field {form} name="category_id">
					<Form.Control>
						{#snippet children({ props })}
							<Form.Label>Category</Form.Label>
							<Select.Root
								allowDeselect={false}
								type="single"
								value={$formData.category_id}
								disabled={isSubmitting || isLoadingCategories}
								onValueChange={(value) => {
									if (typeof value === 'string') {
										$formData.category_id = value;
									}
								}}
							>
								<Select.Trigger {...props} class="w-full">
									<Select.Value placeholder="Select category" />
								</Select.Trigger>
								<Select.Content>
									<Select.Item value={NO_CATEGORY_VALUE} label={t('master.products.noCategory')}>
										No category
									</Select.Item>
									{#each categories as category (category.id)}
										<Select.Item value={String(category.id)} label={category.name}>
											{category.name}
										</Select.Item>
									{/each}
								</Select.Content>
							</Select.Root>
						{/snippet}
					</Form.Control>
					<Form.FieldErrors />
				</Form.Field>

				<Form.Field {form} name="is_active">
					<Form.Control>
						{#snippet children({ props })}
							<Form.Label>Status</Form.Label>
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
									<Select.Value placeholder="Select status" />
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

				{#if categoryErrorMessage}
					<ErrorAlert
						title={t('master.products.errorLoadOptions')}
						message={categoryErrorMessage}
					/>
				{/if}

				{#if errorMessage}
					<ErrorAlert title={t('master.products.errorCreate')} message={errorMessage} />
				{/if}
			</div>

			<Dialog.Footer class="border-t bg-muted/30 px-6 py-4">
				<Dialog.Close
					type="button"
					class={cn(buttonVariants({ variant: 'outline' }))}
					disabled={isSubmitting}
				>
					Cancel
				</Dialog.Close>
				<Button type="submit" disabled={isSubmitting}>
					{#if isSubmitting}
						<Spinner />
					{/if}
					{isSubmitting ? t('master.products.creating') : t('master.products.createBtn')}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>
