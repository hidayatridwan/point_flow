<script lang="ts" module>
	let editFormSequence = 0;

	function getNextEditFormId() {
		editFormSequence += 1;
		return `unit-edit-${editFormSequence}`;
	}
</script>

<script lang="ts">
	import { updateUnit, type Unit } from '$lib/api/units.js';
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

	const editUnitSchema = z.object({
		code: z
			.string()
			.trim()
			.min(1, 'Code is required.')
			.max(20, 'Code must be at most 20 characters.'),
		name: z
			.string()
			.trim()
			.min(1, 'Name is required.')
			.max(100, 'Name must be at most 100 characters.'),
		allow_decimal: z.enum(['true', 'false']),
		is_active: z.enum(['true', 'false'])
	});

	type EditUnitForm = z.infer<typeof editUnitSchema>;

	let {
		unit,
		open = $bindable(false),
		onUpdated
	}: {
		unit: Unit;
		open: boolean;
		onUpdated?: (unit: Unit) => void;
	} = $props();

	let isSubmitting = $state(false);
	let errorMessage = $state<string | null>(null);

	const defaultFormData: EditUnitForm = {
		code: '',
		name: '',
		allow_decimal: 'false',
		is_active: 'true'
	};
	const formId = getNextEditFormId();

	const form = superForm(defaults(defaultFormData, zod4(editUnitSchema)), {
		id: formId,
		SPA: true,
		validationMethod: 'submit-only',
		validators: zod4Client(editUnitSchema)
	});
	const { form: formData, validateForm } = form;

	$effect(() => {
		if (!open) return;
		errorMessage = null;
		$formData = getInitialFormData(unit);
	});

	async function handleSubmit(event: SubmitEvent) {
		event.preventDefault();
		const result = await validateForm({ update: true, focusOnError: true });

		if (!result.valid) return;

		isSubmitting = true;
		errorMessage = null;

		try {
			const updatedUnit = await updateUnit(unit.id, {
				code: result.data.code,
				name: result.data.name,
				allow_decimal: result.data.allow_decimal === 'true',
				is_active: result.data.is_active === 'true'
			});
			onUpdated?.(updatedUnit);
			toast.success(t('master.units.updatedToast'), {
				description: `Changes for ${updatedUnit.name} have been saved.`
			});
			open = false;
		} catch (error) {
			errorMessage = getErrorMessage(error);
		} finally {
			isSubmitting = false;
		}
	}

	function getInitialFormData(unit: Unit): EditUnitForm {
		return {
			code: unit.code,
			name: unit.name,
			allow_decimal: String(unit.allow_decimal) as EditUnitForm['allow_decimal'],
			is_active: String(unit.is_active) as EditUnitForm['is_active']
		};
	}

	function getErrorMessage(error: unknown) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return t('master.units.errorSave');
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="overflow-hidden p-0 sm:max-w-lg">
		<form class="grid gap-6" onsubmit={handleSubmit}>
			<Dialog.Header class="px-6 pt-6">
				<Dialog.Title>Edit Unit</Dialog.Title>
				<Dialog.Description
					>Update measurement unit details and save the changes.</Dialog.Description
				>
			</Dialog.Header>

			<div class="grid gap-4 px-6">
				<Form.Field {form} name="code">
					<Form.Control>
						{#snippet children({ props })}
							<Form.Label>Code</Form.Label>
							<Input
								{...props}
								placeholder="pcs"
								disabled={isSubmitting}
								bind:value={$formData.code}
							/>
						{/snippet}
					</Form.Control>
					<Form.FieldErrors />
				</Form.Field>

				<Form.Field {form} name="name">
					<Form.Control>
						{#snippet children({ props })}
							<Form.Label>Name</Form.Label>
							<Input
								{...props}
								placeholder="Pieces"
								disabled={isSubmitting}
								bind:value={$formData.name}
							/>
						{/snippet}
					</Form.Control>
					<Form.FieldErrors />
				</Form.Field>

				<div class="grid gap-4 sm:grid-cols-2">
					<Form.Field {form} name="allow_decimal">
						<Form.Control>
							{#snippet children({ props })}
								<Form.Label>Decimal Quantity</Form.Label>
								<Select.Root
									allowDeselect={false}
									type="single"
									value={$formData.allow_decimal}
									disabled={isSubmitting}
									onValueChange={(value) => {
										if (value === 'true' || value === 'false') {
											$formData.allow_decimal = value;
										}
									}}
								>
									<Select.Trigger {...props} class="w-full">
										<Select.Value placeholder={t('master.units.selectDecimalRule')} />
									</Select.Trigger>
									<Select.Content>
										<Select.Item value="false" label={t('master.units.decimalWholeOnly')}>
											Whole numbers only
										</Select.Item>
										<Select.Item value="true" label={t('master.units.decimalAllowed')}>Allow decimals</Select.Item>
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
				</div>

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
