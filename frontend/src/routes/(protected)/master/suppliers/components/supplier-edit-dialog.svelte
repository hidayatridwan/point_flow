<script lang="ts" module>
	let editFormSequence = 0;

	function getNextEditFormId() {
		editFormSequence += 1;
		return `supplier-edit-${editFormSequence}`;
	}
</script>

<script lang="ts">
	import { updateSupplier, type Supplier } from '$lib/api/suppliers.js';
	import { ApiError } from '$lib/api/types.js';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import { Button, buttonVariants } from '$lib/components/ui/button/index.js';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import * as Form from '$lib/components/ui/form/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import * as Select from '$lib/components/ui/select/index.js';
	import { Textarea } from '$lib/components/ui/textarea/index.js';
	import { cn } from '$lib/utils.js';
	import { defaults, superForm } from 'sveltekit-superforms';
	import { zod4, zod4Client } from 'sveltekit-superforms/adapters';
	import { z } from 'zod';

	const editSupplierSchema = z.object({
		name: z
			.string()
			.trim()
			.min(1, 'Name is required.')
			.max(150, 'Name must be at most 150 characters.'),
		contact_name: z.string().trim().max(100, 'Contact name must be at most 100 characters.'),
		phone: z.string().trim().max(50, 'Phone must be at most 50 characters.'),
		email: z
			.string()
			.trim()
			.max(150, 'Email must be at most 150 characters.')
			.refine((value) => !value || z.email().safeParse(value).success, {
				message: 'Enter a valid email address.'
			}),
		address: z.string().trim().max(500, 'Address must be at most 500 characters.'),
		is_active: z.enum(['true', 'false'])
	});

	type EditSupplierForm = z.infer<typeof editSupplierSchema>;

	let {
		supplier,
		open = $bindable(false),
		onUpdated
	}: {
		supplier: Supplier;
		open: boolean;
		onUpdated?: (supplier: Supplier) => void;
	} = $props();

	let isSubmitting = $state(false);
	let errorMessage = $state<string | null>(null);

	const defaultFormData: EditSupplierForm = {
		name: '',
		contact_name: '',
		phone: '',
		email: '',
		address: '',
		is_active: 'true'
	};
	const formId = getNextEditFormId();

	const form = superForm(defaults(defaultFormData, zod4(editSupplierSchema)), {
		id: formId,
		SPA: true,
		validationMethod: 'submit-only',
		validators: zod4Client(editSupplierSchema)
	});
	const { form: formData, validateForm } = form;

	$effect(() => {
		if (!open) return;
		errorMessage = null;
		$formData = getInitialFormData(supplier);
	});

	async function handleSubmit(event: SubmitEvent) {
		event.preventDefault();
		const result = await validateForm({ update: true, focusOnError: true });

		if (!result.valid) return;

		isSubmitting = true;
		errorMessage = null;

		try {
			const updatedSupplier = await updateSupplier(supplier.id, {
				name: result.data.name,
				contact_name: emptyToNull(result.data.contact_name),
				phone: emptyToNull(result.data.phone),
				email: emptyToNull(result.data.email),
				address: emptyToNull(result.data.address),
				is_active: result.data.is_active === 'true'
			});
			onUpdated?.(updatedSupplier);
			open = false;
		} catch (error) {
			errorMessage = getErrorMessage(error);
		} finally {
			isSubmitting = false;
		}
	}

	function getInitialFormData(supplier: Supplier): EditSupplierForm {
		return {
			name: supplier.name,
			contact_name: supplier.contact_name ?? '',
			phone: supplier.phone ?? '',
			email: supplier.email ?? '',
			address: supplier.address ?? '',
			is_active: String(supplier.is_active) as EditSupplierForm['is_active']
		};
	}

	function emptyToNull(value: string) {
		const trimmed = value.trim();
		return trimmed ? trimmed : null;
	}

	function getErrorMessage(error: unknown) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return 'Failed to save supplier.';
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="overflow-hidden p-0 sm:max-w-lg">
		<form class="grid gap-6" onsubmit={handleSubmit}>
			<Dialog.Header class="px-6 pt-6">
				<Dialog.Title>Edit Supplier</Dialog.Title>
				<Dialog.Description
					>Update supplier contact details and save the changes.</Dialog.Description
				>
			</Dialog.Header>

			<div class="grid gap-4 px-6">
				<Form.Field {form} name="name">
					<Form.Control>
						{#snippet children({ props })}
							<Form.Label>Name</Form.Label>
							<Input
								{...props}
								placeholder="PT Sumber Barang"
								disabled={isSubmitting}
								bind:value={$formData.name}
							/>
						{/snippet}
					</Form.Control>
					<Form.FieldErrors />
				</Form.Field>

				<div class="grid gap-4 sm:grid-cols-2">
					<Form.Field {form} name="contact_name">
						<Form.Control>
							{#snippet children({ props })}
								<Form.Label>Contact Name</Form.Label>
								<Input
									{...props}
									placeholder="Budi"
									disabled={isSubmitting}
									bind:value={$formData.contact_name}
								/>
							{/snippet}
						</Form.Control>
						<Form.FieldErrors />
					</Form.Field>

					<Form.Field {form} name="phone">
						<Form.Control>
							{#snippet children({ props })}
								<Form.Label>Phone</Form.Label>
								<Input
									{...props}
									placeholder="08123456789"
									disabled={isSubmitting}
									bind:value={$formData.phone}
								/>
							{/snippet}
						</Form.Control>
						<Form.FieldErrors />
					</Form.Field>
				</div>

				<Form.Field {form} name="email">
					<Form.Control>
						{#snippet children({ props })}
							<Form.Label>Email</Form.Label>
							<Input
								{...props}
								type="email"
								placeholder="supplier@example.com"
								disabled={isSubmitting}
								bind:value={$formData.email}
							/>
						{/snippet}
					</Form.Control>
					<Form.FieldErrors />
				</Form.Field>

				<Form.Field {form} name="address">
					<Form.Control>
						{#snippet children({ props })}
							<Form.Label>Address</Form.Label>
							<Textarea
								{...props}
								placeholder="Enter supplier address"
								disabled={isSubmitting}
								bind:value={$formData.address}
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
					{isSubmitting ? 'Saving...' : 'Save changes'}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>
