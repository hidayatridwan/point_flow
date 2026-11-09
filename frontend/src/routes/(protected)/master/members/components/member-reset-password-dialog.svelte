<script lang="ts">
	import { resetMemberPassword, type Member } from '$lib/api/members.js';
	import { ApiError } from '$lib/api/types.js';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import { Button, buttonVariants } from '$lib/components/ui/button/index.js';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import * as Form from '$lib/components/ui/form/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Spinner } from '$lib/components/ui/spinner/index.js';
	import { cn } from '$lib/utils.js';
	import { toast } from 'svelte-sonner';
	import { t } from '$lib/i18n/locale.svelte.js';
	import { defaults, superForm } from 'sveltekit-superforms';
	import { zod4, zod4Client } from 'sveltekit-superforms/adapters';
	import { z } from 'zod';

	const resetPasswordSchema = z.object({
		new_password: z
			.string()
			.min(8, 'Password must contain at least 8 characters.')
			.max(200, 'Password must be at most 200 characters.')
	});

	type ResetPasswordForm = z.infer<typeof resetPasswordSchema>;

	let {
		member,
		open = $bindable(false)
	}: {
		member: Member;
		open: boolean;
	} = $props();

	let isSubmitting = $state(false);
	let errorMessage = $state<string | null>(null);

	const defaultFormData: ResetPasswordForm = { new_password: '' };

	const form = superForm(defaults(defaultFormData, zod4(resetPasswordSchema)), {
		id: 'member-reset-password',
		SPA: true,
		validationMethod: 'submit-only',
		validators: zod4Client(resetPasswordSchema)
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
			await resetMemberPassword(member.id, { new_password: result.data.new_password });
			toast.success(t('master.members.resetToast'), {
				description: member.full_name
			});
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
		return t('master.members.errorReset');
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="p-0 sm:max-w-md">
		<form class="grid gap-6" onsubmit={handleSubmit}>
			<Dialog.Header class="px-6 pt-6">
				<Dialog.Title>{t('master.members.resetTitle')}</Dialog.Title>
				<Dialog.Description>
					{t('master.members.resetDesc').replace('{name}', member.full_name)}
				</Dialog.Description>
			</Dialog.Header>

			<div class="grid gap-4 px-6">
				<Form.Field {form} name="new_password">
					<Form.Control>
						{#snippet children({ props })}
							<Form.Label>{t('master.members.initialPassword')}</Form.Label>
							<Input
								{...props}
								type="password"
								autocomplete="new-password"
								disabled={isSubmitting}
								bind:value={$formData.new_password}
							/>
						{/snippet}
					</Form.Control>
					<Form.Description>{t('master.members.passwordHint')}</Form.Description>
					<Form.FieldErrors />
				</Form.Field>

				{#if errorMessage}
					<ErrorAlert title={t('master.members.errorReset')} message={errorMessage} />
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
					{isSubmitting ? t('master.members.resetting') : t('master.members.resetBtn')}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>
