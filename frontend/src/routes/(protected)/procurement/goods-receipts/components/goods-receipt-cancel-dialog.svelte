<script lang="ts">
	import BanIcon from '@lucide/svelte/icons/ban';
	import { cancelGoodsReceipt, type GoodsReceipt } from '$lib/api/goods-receipts.js';
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
		goodsReceipt,
		open = $bindable(false),
		onCancelled
	}: {
		goodsReceipt: GoodsReceipt;
		open: boolean;
		onCancelled?: (goodsReceipt: GoodsReceipt) => void | Promise<void>;
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
			const cancelledGoodsReceipt = await cancelGoodsReceipt(goodsReceipt.id, {
				reason: emptyToNull(reason)
			});
			await onCancelled?.(cancelledGoodsReceipt);
			toast.success(t('procurement.goodsReceipts.cancel.cancelledToast'), {
				description: `${cancelledGoodsReceipt.gr_no} was cancelled.`
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
		return t('procurement.goodsReceipts.cancel.cancelFailed');
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="overflow-hidden p-0 sm:max-w-lg">
		<form class="grid gap-6" onsubmit={handleSubmit}>
			<Dialog.Header class="px-6 pt-6">
				<Dialog.Title>Cancel Goods Receipt</Dialog.Title>
				<Dialog.Description>
					Cancel {goodsReceipt.gr_no}. Add a reason if this needs audit context.
				</Dialog.Description>
			</Dialog.Header>

			<div class="grid gap-4 px-6">
				<div class="space-y-2">
					<label for={`goods-receipt-cancel-reason-${goodsReceipt.id}`} class="text-sm font-medium">
						Cancel Reason
					</label>
					<Textarea
						id={`goods-receipt-cancel-reason-${goodsReceipt.id}`}
						placeholder={t('procurement.goodsReceipts.cancel.reasonPlaceholder')}
						disabled={isSubmitting}
						bind:value={reason}
					/>
				</div>

				{#if errorMessage}
					<ErrorAlert
						title={t('procurement.goodsReceipts.cancel.cancelFailed')}
						message={errorMessage}
					/>
				{/if}
			</div>

			<Dialog.Footer class="border-t bg-muted/30 px-6 py-4">
				<Dialog.Close
					type="button"
					class={cn(buttonVariants({ variant: 'outline' }))}
					disabled={isSubmitting}
				>
					Keep receipt
				</Dialog.Close>
				<Button type="submit" variant="destructive" disabled={isSubmitting}>
					{#if isSubmitting}
						<Spinner />
					{:else}
						<BanIcon />
					{/if}
					{isSubmitting
						? t('procurement.goodsReceipts.cancel.cancelling')
						: t('procurement.goodsReceipts.cancel.cancelBtn')}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>
