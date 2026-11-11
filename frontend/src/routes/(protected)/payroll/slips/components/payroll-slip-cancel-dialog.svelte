<script lang="ts">
	import BanIcon from '@lucide/svelte/icons/ban';
	import { cancelPayrollSlip, type PayrollSlip } from '$lib/api/payroll.js';
	import { ApiError } from '$lib/api/types.js';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import { Button, buttonVariants } from '$lib/components/ui/button/index.js';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Spinner } from '$lib/components/ui/spinner/index.js';
	import { Textarea } from '$lib/components/ui/textarea/index.js';
	import { cn } from '$lib/utils.js';
	import { toast } from 'svelte-sonner';
	import { t } from '$lib/i18n/locale.svelte.js';

	let {
		payrollSlip,
		open = $bindable(false),
		onCancelled
	}: {
		payrollSlip: PayrollSlip;
		open: boolean;
		onCancelled?: (payrollSlip: PayrollSlip) => void | Promise<void>;
	} = $props();

	let reason = $state('');
	let isSubmitting = $state(false);
	let errorMessage = $state<string | null>(null);

	$effect(() => {
		if (!open) return;
		reason = '';
		errorMessage = null;
	});

	async function handleSubmit(event: SubmitEvent) {
		event.preventDefault();
		if (isSubmitting) return;

		isSubmitting = true;
		errorMessage = null;

		try {
			const cancelledPayrollSlip = await cancelPayrollSlip(payrollSlip.id, {
				reason: emptyToNull(reason)
			});
			await onCancelled?.(cancelledPayrollSlip);
			toast.success(t('payroll.slips.cancel.cancelledToast'), {
				description: `${cancelledPayrollSlip.slip_no} ${t('common.wasCancelled')}`
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
		return t('payroll.slips.cancel.cancelFailed');
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="p-0 sm:max-w-lg">
		<form class="grid gap-6" onsubmit={handleSubmit}>
			<Dialog.Header class="px-6 pt-6">
				<Dialog.Title>{t('payroll.slips.cancel.title')}</Dialog.Title>
				<Dialog.Description>
					{t('payroll.slips.cancel.desc').replace('{no}', payrollSlip.slip_no)}
				</Dialog.Description>
			</Dialog.Header>

			<div class="grid gap-4 px-6">
				<div class="space-y-2">
					<label for={`payroll-slip-cancel-reason-${payrollSlip.id}`} class="text-sm font-medium">
						{t('payroll.slips.cancel.cancelReason')}
					</label>
					<Textarea
						id={`payroll-slip-cancel-reason-${payrollSlip.id}`}
						placeholder={t('payroll.slips.cancel.reasonPlaceholder')}
						disabled={isSubmitting}
						bind:value={reason}
					/>
				</div>

				{#if errorMessage}
					<ErrorAlert title={t('payroll.slips.cancel.cancelFailed')} message={errorMessage} />
				{/if}
			</div>

			<Dialog.Footer class="border-t bg-muted/30 px-6 py-4">
				<Dialog.Close
					type="button"
					class={cn(buttonVariants({ variant: 'outline' }))}
					disabled={isSubmitting}
				>
					{t('payroll.slips.cancel.keepSlip')}
				</Dialog.Close>
				<Button type="submit" variant="destructive" disabled={isSubmitting}>
					{#if isSubmitting}
						<Spinner />
					{:else}
						<BanIcon />
					{/if}
					{isSubmitting
						? t('payroll.slips.cancel.cancelling')
						: t('payroll.slips.cancel.cancelBtn')}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>
