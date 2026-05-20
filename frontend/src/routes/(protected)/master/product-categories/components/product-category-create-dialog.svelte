<script lang="ts">
	import CircleAlertIcon from '@lucide/svelte/icons/circle-alert';
	import {
		createProductCategory,
		type ProductCategory
	} from '$lib/api/product-categories.js';
	import { ApiError } from '$lib/api/types.js';
	import * as Alert from '$lib/components/ui/alert/index.js';
	import { Button, buttonVariants } from '$lib/components/ui/button/index.js';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import * as Form from '$lib/components/ui/form/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import * as Select from '$lib/components/ui/select/index.js';
	import { cn } from '$lib/utils.js';
	import { defaults, superForm } from 'sveltekit-superforms';
	import { zod4, zod4Client } from 'sveltekit-superforms/adapters';
	import { z } from 'zod';

	const createCategorySchema = z.object({
		name: z.string().trim().min(1, 'Name wajib diisi.').max(100, 'Name maksimal 100 karakter.'),
		description: z.string().trim().max(500, 'Description maksimal 500 karakter.'),
		is_active: z.enum(['true', 'false'])
	});

	type CreateCategoryForm = z.infer<typeof createCategorySchema>;

	let {
		open = $bindable(false),
		onCreated
	}: {
		open: boolean;
		onCreated?: (category: ProductCategory) => void | Promise<void>;
	} = $props();

	let isSubmitting = $state(false);
	let errorMessage = $state<string | null>(null);

	const defaultFormData: CreateCategoryForm = {
		name: '',
		description: '',
		is_active: 'true'
	};

	const form = superForm(defaults(defaultFormData, zod4(createCategorySchema)), {
		SPA: true,
		validationMethod: 'submit-only',
		validators: zod4Client(createCategorySchema)
	});
	const { form: formData, validateForm } = form;

	$effect(() => {
		if (!open) return;
		errorMessage = null;
		$formData = { ...defaultFormData };
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
			const createdCategory = await createProductCategory({
				name: result.data.name,
				description: result.data.description,
				is_active: result.data.is_active === 'true'
			});
			await onCreated?.(createdCategory);
			open = false;
		} catch (error) {
			errorMessage = getErrorMessage(error);
		} finally {
			isSubmitting = false;
		}
	}

	function getErrorMessage(error: unknown) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return 'Gagal membuat product category.';
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="overflow-hidden p-0 sm:max-w-lg">
		<form class="grid gap-6" onsubmit={handleSubmit}>
			<Dialog.Header class="px-6 pt-6">
				<Dialog.Title>Add Product Category</Dialog.Title>
				<Dialog.Description>
					Buat kategori produk baru untuk membantu pengelompokan item.
				</Dialog.Description>
			</Dialog.Header>

			<div class="grid gap-4 px-6">
				<Form.Field {form} name="name">
					<Form.Control>
						{#snippet children({ props })}
							<Form.Label>Name</Form.Label>
							<Input
								{...props}
								placeholder="Masukkan nama kategori"
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
							<textarea
								{...props}
								class="min-h-24 w-full rounded-md border border-input bg-transparent px-2.5 py-2 text-sm shadow-xs outline-none transition-[color,box-shadow] placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 aria-invalid:border-destructive aria-invalid:ring-3 aria-invalid:ring-destructive/20 dark:bg-input/30 dark:aria-invalid:border-destructive/50 dark:aria-invalid:ring-destructive/40"
								placeholder="Masukkan deskripsi kategori"
								disabled={isSubmitting}
								bind:value={$formData.description}
							></textarea>
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
									<Select.Value placeholder="Pilih status" />
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

				{#if errorMessage}
					<Alert.Root variant="destructive">
						<CircleAlertIcon />
						<Alert.Title>Gagal membuat category.</Alert.Title>
						<Alert.Description>{errorMessage}</Alert.Description>
					</Alert.Root>
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
					{isSubmitting ? 'Creating...' : 'Create category'}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>
