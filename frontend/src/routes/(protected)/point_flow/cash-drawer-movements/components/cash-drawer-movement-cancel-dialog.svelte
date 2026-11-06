<script lang="ts">
	import BanIcon from '@lucide/svelte/icons/ban';
	import { cancelCashDrawerMovement, type CashDrawerMovement } from '$lib/api/cash-registers.js';
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
		movement,
		open = $bindable(false),
		onCancelled
	}: {
		movement: CashDrawerMovement;
		open: boolean;
		onCancelled?: (movement: CashDrawerMovement) => void | Promise<void>;
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
			const cancelledMovement = await cancelCashDrawerMovement(movement.id, {
				reason: emptyToNull(reason)
			});
			await onCancelled?.(cancelledMovement);
			toast.success(t('point_flow.cashMovements.cancel.cancelledToast'), {
				description: `${cancelledMovement.movement_no} ${t('common.wasCancelled')}`
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
		return t('point_flow.cashMovements.cancel.cancelFailed');
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="p-0 sm:max-w-lg">
		<form class="grid gap-6" onsubmit={handleSubmit}>
			<Dialog.Header class="px-6 pt-6">
				<Dialog.Title>{t('point_flow.cashMovements.cancel.title')}</Dialog.Title>
				<Dialog.Description>
					{t('point_flow.cashMovements.cancel.desc').replace('{no}', movement.movement_no)}
				</Dialog.Description>
			</Dialog.Header>

			<div class="grid gap-4 px-6">
				<div class="space-y-2">
					<label for={`cash-movement-cancel-reason-${movement.id}`} class="text-sm font-medium">
						{t('point_flow.cashMovements.cancel.cancelReason')}
					</label>
					<Textarea
						id={`cash-movement-cancel-reason-${movement.id}`}
						placeholder={t('point_flow.cashMovements.cancel.reasonPlaceholder')}
						disabled={isSubmitting}
						bind:value={reason}
					/>
				</div>

				{#if errorMessage}
					<ErrorAlert title={t('point_flow.cashMovements.cancel.cancelFailed')} message={errorMessage} />
				{/if}
			</div>

			<Dialog.Footer class="border-t bg-muted/30 px-6 py-4">
				<Dialog.Close
					type="button"
					class={cn(buttonVariants({ variant: 'outline' }))}
					disabled={isSubmitting}
				>
					{t('point_flow.cashMovements.cancel.keepMovement')}
				</Dialog.Close>
				<Button type="submit" variant="destructive" disabled={isSubmitting}>
					{#if isSubmitting}
						<Spinner />
					{:else}
						<BanIcon />
					{/if}
					{isSubmitting
						? t('point_flow.cashMovements.cancel.cancelling')
						: t('point_flow.cashMovements.cancel.cancelBtn')}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>
