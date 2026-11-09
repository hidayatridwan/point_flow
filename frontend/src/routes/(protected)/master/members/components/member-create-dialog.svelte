<script lang="ts">
	import { createMember, type Member } from '$lib/api/members.js';
	import { ApiError } from '$lib/api/types.js';
	import { generateStrongPassword } from '$lib/password.js';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import RefreshCwIcon from '@lucide/svelte/icons/refresh-cw';
	import { Button, buttonVariants } from '$lib/components/ui/button/index.js';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import * as Form from '$lib/components/ui/form/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import * as Select from '$lib/components/ui/select/index.js';
	import { Spinner } from '$lib/components/ui/spinner/index.js';
	import { Textarea } from '$lib/components/ui/textarea/index.js';
	import { cn } from '$lib/utils.js';
	import { toast } from 'svelte-sonner';
	import { t } from '$lib/i18n/locale.svelte.js';
	import { defaults, superForm } from 'sveltekit-superforms';
	import { zod4, zod4Client } from 'sveltekit-superforms/adapters';
	import { z } from 'zod';

	const normalizeStringInput = (value: unknown) => {
		if (value === null || value === undefined) return '';
		return String(value);
	};

	const optionalEmail = z.preprocess(
		normalizeStringInput,
		z
			.string()
			.trim()
			.max(160, 'Email must be at most 160 characters.')
			.refine((value) => value === '' || value.includes('@'), {
				message: 'Email must be a valid email.'
			})
	);

	const createMemberSchema = z.object({
		full_name: z
			.string()
			.trim()
			.min(1, 'Full name is required.')
			.max(160, 'Full name must be at most 160 characters.'),
		phone: z
			.string()
			.trim()
			.min(1, 'Phone is required.')
			.max(30, 'Phone must be at most 30 characters.'),
		email: optionalEmail,
		password: z
			.string()
			.min(8, 'Password must contain at least 8 characters.')
			.max(200, 'Password must be at most 200 characters.'),
		address: z.string().trim().max(500, 'Address must be at most 500 characters.'),
		notes: z.string().trim().max(2000, 'Notes must be at most 2000 characters.'),
		is_active: z.enum(['true', 'false'])
	});

	type CreateMemberForm = z.infer<typeof createMemberSchema>;

	let {
		open = $bindable(false),
		onCreated
	}: {
		open: boolean;
		onCreated?: (member: Member) => void | Promise<void>;
	} = $props();

	let isSubmitting = $state(false);
	let errorMessage = $state<string | null>(null);

	const defaultFormData: CreateMemberForm = {
		full_name: '',
		phone: '',
		email: '',
		password: '',
		address: '',
		notes: '',
		is_active: 'true'
	};

	const form = superForm(defaults(defaultFormData, zod4(createMemberSchema)), {
		id: 'member-create',
		SPA: true,
		validationMethod: 'submit-only',
		validators: zod4Client(createMemberSchema)
	});
	const { form: formData, validateForm } = form;

	$effect(() => {
		if (!open) return;
		errorMessage = null;
		$formData = { ...defaultFormData, password: generateStrongPassword() };
	});

	function regeneratePassword() {
		$formData.password = generateStrongPassword();
	}

	async function handleSubmit(event: SubmitEvent) {
		event.preventDefault();
		const result = await validateForm({ update: true, focusOnError: true });

		if (!result.valid) return;

		isSubmitting = true;
		errorMessage = null;

		try {
			const createdMember = await createMember({
				full_name: result.data.full_name,
				phone: result.data.phone,
				email: emptyToNull(result.data.email),
				password: result.data.password,
				address: emptyToNull(result.data.address),
				notes: emptyToNull(result.data.notes),
				is_active: result.data.is_active === 'true'
			});
			await onCreated?.(createdMember);
			toast.success(t('master.members.createdToast'), {
				description: `${createdMember.full_name} ${t('common.wasAdded')}`
			});
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
		return t('master.members.errorCreate');
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="p-0 sm:max-w-lg">
		<form class="grid gap-6" onsubmit={handleSubmit}>
			<Dialog.Header class="px-6 pt-6">
				<Dialog.Title>{t('master.members.addTitle')}</Dialog.Title>
				<Dialog.Description>{t('master.members.addDesc')}</Dialog.Description>
			</Dialog.Header>

			<div class="grid gap-4 px-6">
				<Form.Field {form} name="full_name">
					<Form.Control>
						{#snippet children({ props })}
							<Form.Label>{t('common.name')}</Form.Label>
							<Input
								{...props}
								placeholder="Siti Aminah"
								disabled={isSubmitting}
								bind:value={$formData.full_name}
							/>
						{/snippet}
					</Form.Control>
					<Form.FieldErrors />
				</Form.Field>

				<div class="grid gap-4 sm:grid-cols-2">
					<Form.Field {form} name="phone">
						<Form.Control>
							{#snippet children({ props })}
								<Form.Label>{t('common.phone')}</Form.Label>
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

					<Form.Field {form} name="email">
						<Form.Control>
							{#snippet children({ props })}
								<Form.Label>{t('common.email')}</Form.Label>
								<Input
									{...props}
									type="email"
									placeholder="siti@example.com"
									disabled={isSubmitting}
									bind:value={$formData.email}
								/>
							{/snippet}
						</Form.Control>
						<Form.FieldErrors />
					</Form.Field>
				</div>

				<Form.Field {form} name="password">
					<Form.Control>
						{#snippet children({ props })}
							<Form.Label>{t('master.members.initialPassword')}</Form.Label>
							<Input
								{...props}
								type="text"
								autocomplete="off"
								disabled={isSubmitting}
								bind:value={$formData.password}
							/>
						{/snippet}
					</Form.Control>
					<Form.Description>{t('master.members.passwordHint')}</Form.Description>
					<div>
						<Button
							type="button"
							variant="outline"
							size="sm"
							disabled={isSubmitting}
							onclick={regeneratePassword}
						>
							<RefreshCwIcon />
							{t('master.members.regeneratePassword')}
						</Button>
					</div>
					<Form.FieldErrors />
				</Form.Field>

				<Form.Field {form} name="address">
					<Form.Control>
						{#snippet children({ props })}
							<Form.Label>{t('common.address')}</Form.Label>
							<Textarea
								{...props}
								placeholder={t('master.members.addressPlaceholder')}
								disabled={isSubmitting}
								bind:value={$formData.address}
							/>
						{/snippet}
					</Form.Control>
					<Form.FieldErrors />
				</Form.Field>

				<div class="grid gap-4 sm:grid-cols-2">
					<Form.Field {form} name="is_active">
						<Form.Control>
							{#snippet children({ props })}
								<Form.Label>{t('common.status')}</Form.Label>
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

					<Form.Field {form} name="notes">
						<Form.Control>
							{#snippet children({ props })}
								<Form.Label>{t('common.notes')}</Form.Label>
								<Input
									{...props}
									placeholder={t('master.members.notesPlaceholder')}
									disabled={isSubmitting}
									bind:value={$formData.notes}
								/>
							{/snippet}
						</Form.Control>
						<Form.FieldErrors />
					</Form.Field>
				</div>

				{#if errorMessage}
					<ErrorAlert title={t('master.members.errorCreate')} message={errorMessage} />
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
					{isSubmitting ? t('master.members.creating') : t('master.members.createBtn')}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>
