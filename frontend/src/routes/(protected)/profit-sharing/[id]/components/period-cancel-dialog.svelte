<script lang="ts">
	import { cancelPeriod, type ProfitSharingPeriodDetail } from '$lib/api/profit-sharing.js';
	import { ApiError } from '$lib/api/types.js';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import * as AlertDialog from '$lib/components/ui/alert-dialog/index.js';
	import { Textarea } from '$lib/components/ui/textarea/index.js';
	import { Spinner } from '$lib/components/ui/spinner/index.js';
	import { toast } from 'svelte-sonner';
	import { t } from '$lib/i18n/locale.svelte.js';

	let {
		open = $bindable(false),
		period,
		onCancelled
	}: {
		open: boolean;
		period: ProfitSharingPeriodDetail;
		onCancelled?: () => void | Promise<void>;
	} = $props();

	let reason = $state('');
	let isSubmitting = $state(false);
	let errorMessage = $state<string | null>(null);

	$effect(() => {
		if (!open) return;
		reason = '';
		errorMessage = null;
	});

	async function handleConfirm(event: MouseEvent) {
		event.preventDefault();
		if (isSubmitting || !reason.trim()) return;

		isSubmitting = true;
		errorMessage = null;

		try {
			await cancelPeriod(period.id, { reason: reason.trim() });
			await onCancelled?.();
			toast.success(t('finances.profitSharing.cancelledToast'));
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
		return t('finances.profitSharing.errorCancel');
	}
</script>

<AlertDialog.Root bind:open>
	<AlertDialog.Content>
		<AlertDialog.Header>
			<AlertDialog.Title>{t('finances.profitSharing.cancelTitle')}</AlertDialog.Title>
			<AlertDialog.Description>
				{t('finances.profitSharing.cancelDesc').replace('{name}', period.period_name)}
			</AlertDialog.Description>
		</AlertDialog.Header>

		<div class="grid gap-2 px-6">
			<label class="text-sm font-medium" for="cancel-reason">
				{t('finances.profitSharing.cancelReasonLabel')}
			</label>
			<Textarea
				id="cancel-reason"
				placeholder={t('finances.profitSharing.cancelReasonPlaceholder')}
				disabled={isSubmitting}
				bind:value={reason}
			/>
			{#if errorMessage}
				<ErrorAlert title={t('finances.profitSharing.errorCancel')} message={errorMessage} />
			{/if}
		</div>

		<AlertDialog.Footer>
			<AlertDialog.Cancel disabled={isSubmitting}>{t('common.cancel')}</AlertDialog.Cancel>
			<AlertDialog.Action
				variant="destructive"
				disabled={isSubmitting || !reason.trim()}
				onclick={handleConfirm}
			>
				{#if isSubmitting}
					<Spinner />
				{/if}
				{isSubmitting
					? t('finances.profitSharing.cancelling')
					: t('finances.profitSharing.cancelBtn')}
			</AlertDialog.Action>
		</AlertDialog.Footer>
	</AlertDialog.Content>
</AlertDialog.Root>
