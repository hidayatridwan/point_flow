<script lang="ts" module>
	let editFormSequence = 0;

	function getNextEditFormId() {
		editFormSequence += 1;
		return `product-category-edit-${editFormSequence}`;
	}
</script>

<script lang="ts">
	import { updateProductCategory, type ProductCategory } from '$lib/api/product-categories.js';
	import { ApiError } from '$lib/api/types.js';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import { Button, buttonVariants } from '$lib/components/ui/button/index.js';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import * as Form from '$lib/components/ui/form/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import * as Select from '$lib/components/ui/select/index.js';
	import { Spinner } from '$lib/components/ui/spinner/index.js';
	import { Textarea } from '$lib/components/ui/textarea/index.js';
	import { cn } from '$lib/utils.js';
	import { toast } from 'svelte-sonner';
	import { defaults, superForm } from 'sveltekit-superforms';
	import { zod4, zod4Client } from 'sveltekit-superforms/adapters';
	import { z } from 'zod';
	import { t } from '$lib/i18n/locale.svelte.js';

	const editCategorySchema = z.object({
		name: z
			.string()
			.trim()
			.min(1, 'Name is required.')
			.max(100, 'Name must be at most 100 characters.'),
		description: z.string().trim().max(500, 'Description must be at most 500 characters.'),
		is_active: z.enum(['true', 'false'])
	});

	type EditCategoryForm = z.infer<typeof editCategorySchema>;

	let {
		category,
		open = $bindable(false),
		onUpdated
	}: {
		category: ProductCategory;
		open: boolean;
		onUpdated?: (category: ProductCategory) => void;
	} = $props();

	let isSubmitting = $state(false);
	let errorMessage = $state<string | null>(null);

	const defaultFormData: EditCategoryForm = {
		name: '',
		description: '',
		is_active: 'true'
	};
	const formId = getNextEditFormId();

	const form = superForm(defaults(defaultFormData, zod4(editCategorySchema)), {
		id: formId,
		SPA: true,
		validationMethod: 'submit-only',
		validators: zod4Client(editCategorySchema)
	});
	const { form: formData, validateForm } = form;

	$effect(() => {
		if (!open) return;
		errorMessage = null;
		$formData = getInitialFormData(category);
	});

	async function handleSubmit(event: SubmitEvent) {
		event.preventDefault();
		const result = await validateForm({ update: true, focusOnError: true });

		if (!result.valid) {
			return;
		}

		isSubmitting = true;
		errorMessage = null;

		try {
			const updatedCategory = await updateProductCategory(category.id, {
				name: result.data.name,
				description: result.data.description,
				is_active: result.data.is_active === 'true'
			});
			onUpdated?.(updatedCategory);
			toast.success(t('master.productCategories.updatedToast'), {
				description: `Changes for ${updatedCategory.name} have been saved.`
			});
			open = false;
		} catch (error) {
			errorMessage = getErrorMessage(error);
		} finally {
			isSubmitting = false;
		}
	}

	function getInitialFormData(category: ProductCategory): EditCategoryForm {
		return {
			name: category.name,
			description: category.description ?? '',
			is_active: String(category.is_active) as EditCategoryForm['is_active']
		};
	}

	function getErrorMessage(error: unknown) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return t('master.productCategories.errorSave');
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="overflow-hidden p-0 sm:max-w-lg">
		<form class="grid gap-6" onsubmit={handleSubmit}>
			<Dialog.Header class="px-6 pt-6">
				<Dialog.Title>Edit Product Category</Dialog.Title>
				<Dialog.Description>
					Update product category details, then save the changes to the server.
				</Dialog.Description>
			</Dialog.Header>

			<div class="grid gap-4 px-6">
				<Form.Field {form} name="name">
					<Form.Control>
						{#snippet children({ props })}
							<Form.Label>Name</Form.Label>
							<Input
								{...props}
								placeholder={t('master.productCategories.namePlaceholder')}
								disabled={isSubmitting}
								bind:value={$formData.name}
							/>
						{/snippet}
					</Form.Control>
					<Form.FieldErrors />
				</Form.Field>

				<Form.Field {form} name="description">
					<Form.Control>
						{#snippet children({ props })}
							<Form.Label>Description</Form.Label>
							<Textarea
								{...props}
								placeholder={t('master.productCategories.descriptionPlaceholder')}
								disabled={isSubmitting}
								bind:value={$formData.description}
							/>
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

				{#if errorMessage}
					<ErrorAlert title={t('common.saveFailed')} message={errorMessage} />
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
					{isSubmitting ? t('common.saving') : t('common.save')}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>
