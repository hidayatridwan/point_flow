<script lang="ts">
	import { createEmployee, type Employee } from '$lib/api/employees.js';
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

	const salarySchema = z
		.string()
		.trim()
		.min(1, 'Base salary is required.')
		.refine((value) => Number.isFinite(Number(value)), {
			message: 'Base salary must be a valid number.'
		})
		.refine((value) => Number(value) >= 0, {
			message: 'Base salary must be zero or greater.'
		});

	const createEmployeeSchema = z.object({
		name: z
			.string()
			.trim()
			.min(1, 'Name is required.')
			.max(150, 'Name must be at most 150 characters.'),
		position: z
			.string()
			.trim()
			.min(1, 'Position is required.')
			.max(100, 'Position must be at most 100 characters.'),
		phone: z.string().trim().max(50, 'Phone must be at most 50 characters.'),
		address: z.string().trim().max(500, 'Address must be at most 500 characters.'),
		base_salary: salarySchema,
		is_active: z.enum(['true', 'false'])
	});

	type CreateEmployeeForm = z.infer<typeof createEmployeeSchema>;

	let {
		open = $bindable(false),
		onCreated
	}: {
		open: boolean;
		onCreated?: (employee: Employee) => void | Promise<void>;
	} = $props();

	let isSubmitting = $state(false);
	let errorMessage = $state<string | null>(null);

	const defaultFormData: CreateEmployeeForm = {
		name: '',
		position: '',
		phone: '',
		address: '',
		base_salary: '',
		is_active: 'true'
	};

	const form = superForm(defaults(defaultFormData, zod4(createEmployeeSchema)), {
		id: 'employee-create',
		SPA: true,
		validationMethod: 'submit-only',
		validators: zod4Client(createEmployeeSchema)
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

		if (!result.valid) return;

		isSubmitting = true;
		errorMessage = null;

		try {
			const createdEmployee = await createEmployee({
				name: result.data.name,
				position: result.data.position,
				phone: emptyToNull(result.data.phone),
				address: emptyToNull(result.data.address),
				base_salary: Number(result.data.base_salary),
				is_active: result.data.is_active === 'true'
			});
			await onCreated?.(createdEmployee);
			open = false;
		} catch (error) {
			errorMessage = getErrorMessage(error);
		} finally {
			isSubmitting = false;
		}
	}

	function emptyToNull(value: string) {
		const trimmed = value.trim();
		return trimmed ? trimmed : null;
	}

	function getErrorMessage(error: unknown) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return 'Failed to create employee.';
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="overflow-hidden p-0 sm:max-w-lg">
		<form class="grid gap-6" onsubmit={handleSubmit}>
			<Dialog.Header class="px-6 pt-6">
				<Dialog.Title>Add Employee</Dialog.Title>
				<Dialog.Description
					>Create an employee record for operations and payroll.</Dialog.Description
				>
			</Dialog.Header>

			<div class="grid gap-4 px-6">
				<Form.Field {form} name="name">
					<Form.Control>
						{#snippet children({ props })}
							<Form.Label>Name</Form.Label>
							<Input
								{...props}
								placeholder="Siti Aminah"
								disabled={isSubmitting}
								bind:value={$formData.name}
							/>
						{/snippet}
					</Form.Control>
					<Form.FieldErrors />
				</Form.Field>

				<div class="grid gap-4 sm:grid-cols-2">
					<Form.Field {form} name="position">
						<Form.Control>
							{#snippet children({ props })}
								<Form.Label>Position</Form.Label>
								<Input
									{...props}
									placeholder="Cashier"
									disabled={isSubmitting}
									bind:value={$formData.position}
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
									placeholder="08123456780"
									disabled={isSubmitting}
									bind:value={$formData.phone}
								/>
							{/snippet}
						</Form.Control>
						<Form.FieldErrors />
					</Form.Field>
				</div>

				<Form.Field {form} name="base_salary">
					<Form.Control>
						{#snippet children({ props })}
							<Form.Label>Base Salary</Form.Label>
							<Input
								{...props}
								type="number"
								min="0"
								step="1000"
								placeholder="3000000"
								disabled={isSubmitting}
								bind:value={$formData.base_salary}
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
								placeholder="Enter employee address"
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
					<ErrorAlert title="Failed to create employee." message={errorMessage} />
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
					{isSubmitting ? 'Creating...' : 'Create employee'}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>
