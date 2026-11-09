<script lang="ts">
	import { finalizePeriod, type ProfitSharingPeriodDetail } from '$lib/api/profit-sharing.js';
	import { ApiError } from '$lib/api/types.js';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import * as AlertDialog from '$lib/components/ui/alert-dialog/index.js';
	import { Spinner } from '$lib/components/ui/spinner/index.js';
	import { toast } from 'svelte-sonner';
	import { t } from '$lib/i18n/locale.svelte.js';

	let {
		open = $bindable(false),
		period,
		onFinalized
	}: {
		open: boolean;
		period: ProfitSharingPeriodDetail;
		onFinalized?: () => void | Promise<void>;
	} = $props();

	let isSubmitting = $state(false);
	let errorMessage = $state<string | null>(null);

	$effect(() => {
		if (!open) return;
		errorMessage = null;
	});

	async function handleConfirm(event: MouseEvent) {
		event.preventDefault();
		if (isSubmitting) return;

		isSubmitting = true;
		errorMessage = null;

		try {
			await finalizePeriod(period.id);
			await onFinalized?.();
			toast.success(t('finances.profitSharing.finalizedToast'));
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
		return t('finances.profitSharing.errorFinalize');
	}
</script>

<AlertDialog.Root bind:open>
	<AlertDialog.Content>
		<AlertDialog.Header>
			<AlertDialog.Title>{t('finances.profitSharing.finalizeTitle')}</AlertDialog.Title>
			<AlertDialog.Description>
				{t('finances.profitSharing.finalizeDesc').replace('{count}', String(period.member_count))}
			</AlertDialog.Description>
		</AlertDialog.Header>

		{#if errorMessage}
			<div class="px-6">
				<ErrorAlert title={t('finances.profitSharing.errorFinalize')} message={errorMessage} />
			</div>
		{/if}

		<AlertDialog.Footer>
			<AlertDialog.Cancel disabled={isSubmitting}>{t('common.cancel')}</AlertDialog.Cancel>
			<AlertDialog.Action disabled={isSubmitting} onclick={handleConfirm}>
				{#if isSubmitting}
					<Spinner />
				{/if}
				{isSubmitting
					? t('finances.profitSharing.finalizing')
					: t('finances.profitSharing.finalizeBtn')}
			</AlertDialog.Action>
		</AlertDialog.Footer>
	</AlertDialog.Content>
</AlertDialog.Root>
