<script lang="ts" module>
	let editFormSequence = 0;

	function getNextEditFormId() {
		editFormSequence += 1;
		return `product-edit-${editFormSequence}`;
	}
</script>

<script lang="ts">
	import { listProductCategories, type ProductCategory } from '$lib/api/product-categories.js';
	import { updateProduct, type Product } from '$lib/api/products.js';
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

	const NO_CATEGORY_VALUE = 'none';

	const editProductSchema = z.object({
		name: z
			.string()
			.trim()
			.min(1, 'Name is required.')
			.max(160, 'Name must be at most 160 characters.'),
		category_id: z.string(),
		is_active: z.enum(['true', 'false'])
	});

	type EditProductForm = z.infer<typeof editProductSchema>;

	let {
		product,
		open = $bindable(false),
		onUpdated
	}: {
		product: Product;
		open: boolean;
		onUpdated?: (product: Product) => void;
	} = $props();

	let categories = $state<ProductCategory[]>([]);
	let isLoadingCategories = $state(false);
	let isSubmitting = $state(false);
	let errorMessage = $state<string | null>(null);
	let categoryErrorMessage = $state<string | null>(null);
	let latestCategoryRequestId = 0;

	const defaultFormData: EditProductForm = {
		name: '',
		category_id: NO_CATEGORY_VALUE,
		is_active: 'true'
	};
	const formId = getNextEditFormId();

	const form = superForm(defaults(defaultFormData, zod4(editProductSchema)), {
		id: formId,
		SPA: true,
		validationMethod: 'submit-only',
		validators: zod4Client(editProductSchema)
	});
	const { form: formData, validateForm } = form;

	$effect(() => {
		if (!open) return;
		errorMessage = null;
		categoryErrorMessage = null;
		$formData = getInitialFormData(product);
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
			const updatedProduct = await updateProduct(product.id, {
				name: result.data.name,
				category_id: getNullableCategoryId(result.data.category_id),
				is_active: result.data.is_active === 'true'
			});
			onUpdated?.(updatedProduct);
			toast.success('Product updated.', {
				description: `Changes for ${updatedProduct.name} have been saved.`
			});
			open = false;
		} catch (error) {
			errorMessage = getErrorMessage(error, 'Failed to save product.');
		} finally {
			isSubmitting = false;
		}
	}

	function getInitialFormData(product: Product): EditProductForm {
		return {
			name: product.name,
			category_id: product.category_id ? String(product.category_id) : NO_CATEGORY_VALUE,
			is_active: String(product.is_active) as EditProductForm['is_active']
		};
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
				<Dialog.Title>Edit Product</Dialog.Title>
				<Dialog.Description
					>Update product master details, then save the changes.</Dialog.Description
				>
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
									<Select.Item value={NO_CATEGORY_VALUE} label="No category">
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
									<Select.Item value="true" label="Active">Active</Select.Item>
									<Select.Item value="false" label="Inactive">Inactive</Select.Item>
								</Select.Content>
							</Select.Root>
						{/snippet}
					</Form.Control>
					<Form.FieldErrors />
				</Form.Field>

				{#if categoryErrorMessage}
					<ErrorAlert title="Failed to load categories." message={categoryErrorMessage} />
				{/if}

				{#if errorMessage}
					<ErrorAlert title="Save failed." message={errorMessage} />
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
					{isSubmitting ? 'Saving...' : 'Save changes'}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>
