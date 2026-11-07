<script lang="ts">
	import BanIcon from '@lucide/svelte/icons/ban';
	import {
		cancelOnlineOrder,
		type OnlineOrder,
		type OnlineOrderDetail
	} from '$lib/api/online-orders.js';
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
		order,
		open = $bindable(false),
		onDone
	}: {
		order: OnlineOrder;
		open: boolean;
		onDone?: (detail: OnlineOrderDetail) => void | Promise<void>;
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
			const trimmed = reason.trim();
			const detail = await cancelOnlineOrder(order.id, trimmed ? trimmed : null);
			await onDone?.(detail);
			toast.success(t('point_flow.onlineOrders.cancelledToast'), {
				description: `${order.order_no} ${t('common.wasCancelled')}`
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
		return t('point_flow.onlineOrders.cancelFailed');
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="p-0 sm:max-w-lg">
		<form class="grid gap-6" onsubmit={handleSubmit}>
			<Dialog.Header class="px-6 pt-6">
				<Dialog.Title>{t('point_flow.onlineOrders.cancelTitle')}</Dialog.Title>
				<Dialog.Description>
					{t('point_flow.onlineOrders.cancelDesc').replace('{orderNo}', order.order_no)}
				</Dialog.Description>
			</Dialog.Header>

			<div class="grid gap-4 px-6">
				<div class="space-y-2">
					<label for={`online-order-cancel-reason-${order.id}`} class="text-sm font-medium">
						{t('point_flow.onlineOrders.cancelReason')}
					</label>
					<Textarea
						id={`online-order-cancel-reason-${order.id}`}
						placeholder={t('point_flow.onlineOrders.cancelReasonPlaceholder')}
						disabled={isSubmitting}
						bind:value={reason}
					/>
				</div>

				{#if errorMessage}
					<ErrorAlert title={t('point_flow.onlineOrders.cancelFailed')} message={errorMessage} />
				{/if}
			</div>

			<Dialog.Footer class="border-t bg-muted/30 px-6 py-4">
				<Dialog.Close
					type="button"
					class={cn(buttonVariants({ variant: 'outline' }))}
					disabled={isSubmitting}
				>
					{t('point_flow.onlineOrders.keepOrder')}
				</Dialog.Close>
				<Button type="submit" variant="destructive" disabled={isSubmitting}>
					{#if isSubmitting}
						<Spinner />
					{:else}
						<BanIcon />
					{/if}
					{isSubmitting
						? t('point_flow.onlineOrders.cancelling')
						: t('point_flow.onlineOrders.cancelBtn')}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>
