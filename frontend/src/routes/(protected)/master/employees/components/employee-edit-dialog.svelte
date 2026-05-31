<script lang="ts" module>
	let editFormSequence = 0;

	function getNextEditFormId() {
		editFormSequence += 1;
		return `employee-edit-${editFormSequence}`;
	}
</script>

<script lang="ts">
	import { updateEmployee, type Employee } from '$lib/api/employees.js';
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

	const normalizeStringInput = (value: unknown) => {
		if (value === null || value === undefined) return '';
		return String(value);
	};

	const salarySchema = z.preprocess(
		normalizeStringInput,
		z
			.string()
			.trim()
			.min(1, 'Base salary is required.')
			.refine((value) => Number.isFinite(Number(value)), {
				message: 'Base salary must be a valid number.'
			})
			.refine((value) => Number(value) >= 0, {
				message: 'Base salary must be zero or greater.'
			})
	);

	const editEmployeeSchema = z.object({
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

	type EditEmployeeForm = z.infer<typeof editEmployeeSchema>;

	let {
		employee,
		open = $bindable(false),
		onUpdated
	}: {
		employee: Employee;
		open: boolean;
		onUpdated?: (employee: Employee) => void;
	} = $props();

	let isSubmitting = $state(false);
	let errorMessage = $state<string | null>(null);

	const defaultFormData: EditEmployeeForm = {
		name: '',
		position: '',
		phone: '',
		address: '',
		base_salary: '',
		is_active: 'true'
	};
	const formId = getNextEditFormId();

	const form = superForm(defaults(defaultFormData, zod4(editEmployeeSchema)), {
		id: formId,
		SPA: true,
		validationMethod: 'submit-only',
		validators: zod4Client(editEmployeeSchema)
	});
	const { form: formData, validateForm } = form;

	$effect(() => {
		if (!open) return;
		errorMessage = null;
		$formData = getInitialFormData(employee);
	});

	async function handleSubmit(event: SubmitEvent) {
		event.preventDefault();
		const result = await validateForm({ update: true, focusOnError: true });

		if (!result.valid) return;

		isSubmitting = true;
		errorMessage = null;

		try {
			const updatedEmployee = await updateEmployee(employee.id, {
				name: result.data.name,
				position: result.data.position,
				phone: emptyToNull(result.data.phone),
				address: emptyToNull(result.data.address),
				base_salary: Number(result.data.base_salary),
				is_active: result.data.is_active === 'true'
			});
			onUpdated?.(updatedEmployee);
			toast.success(t('master.employees.updatedToast'), {
				description: `Changes for ${updatedEmployee.name} have been saved.`
			});
			open = false;
		} catch (error) {
			errorMessage = getErrorMessage(error);
		} finally {
			isSubmitting = false;
		}
	}

	function getInitialFormData(employee: Employee): EditEmployeeForm {
		return {
			name: employee.name,
			position: employee.position,
			phone: employee.phone ?? '',
			address: employee.address ?? '',
			base_salary: String(employee.base_salary),
			is_active: String(employee.is_active) as EditEmployeeForm['is_active']
		};
	}

	function emptyToNull(value: string) {
		const trimmed = value.trim();
		return trimmed ? trimmed : null;
	}

	function getErrorMessage(error: unknown) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return t('master.employees.errorSave');
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="overflow-hidden p-0 sm:max-w-lg">
		<form class="grid gap-6" onsubmit={handleSubmit}>
			<Dialog.Header class="px-6 pt-6">
				<Dialog.Title>{t('master.employees.editTitle')}</Dialog.Title>
				<Dialog.Description>{t('master.employees.editDesc')}</Dialog.Description>
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
					<ErrorAlert title="Save failed." message={errorMessage} />
				{/if}
			</div>

			<Dialog.Footer class="border-t bg-muted/30 px-6 py-4">
				<Dialog.Close
					type="button"
					class={cn(buttonVariants({ variant: 'outline' }))}
					disabled={isSubmitting}
				>
					{t('common.cancel')}
				</Dialog.Close>
				<Button type="submit" disabled={isSubmitting}>
					{#if isSubmitting}
						<Spinner />
					{/if}
					{isSubmitting ? t('master.employees.saving') : t('master.employees.saveBtn')}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>
