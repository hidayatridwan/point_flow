<script lang="ts">
	import {
		completeOnlineOrder,
		deliverOnlineOrder,
		markOnlineOrderPaid,
		packOnlineOrder,
		readyOnlineOrder,
		type OnlineOrder,
		type OnlineOrderDetail
	} from '$lib/api/online-orders.js';
	import { ApiError } from '$lib/api/types.js';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import { Button, buttonVariants } from '$lib/components/ui/button/index.js';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Spinner } from '$lib/components/ui/spinner/index.js';
	import { cn } from '$lib/utils.js';
	import { toast } from 'svelte-sonner';
	import { t } from '$lib/i18n/locale.svelte.js';

	let {
		order,
		action,
		open = $bindable(false),
		onDone
	}: {
		order: OnlineOrder;
		action: 'pack' | 'ready' | 'deliver' | 'complete' | 'mark-paid';
		open: boolean;
		onDone?: (detail: OnlineOrderDetail) => void | Promise<void>;
	} = $props();

	let isSubmitting = $state(false);
	let errorMessage = $state<string | null>(null);

	const content = $derived(
		(
			{
				pack: {
					title: t('point_flow.onlineOrders.confirmPackTitle'),
					desc: t('point_flow.onlineOrders.confirmPackDesc').replace('{orderNo}', order.order_no),
					confirm: t('point_flow.onlineOrders.actionPack')
				},
				ready: {
					title: t('point_flow.onlineOrders.confirmReadyTitle'),
					desc: t('point_flow.onlineOrders.confirmReadyDesc').replace('{orderNo}', order.order_no),
					confirm: t('point_flow.onlineOrders.actionReady')
				},
				deliver: {
					title: t('point_flow.onlineOrders.confirmDeliverTitle'),
					desc: t('point_flow.onlineOrders.confirmDeliverDesc').replace(
						'{orderNo}',
						order.order_no
					),
					confirm: t('point_flow.onlineOrders.actionDeliver')
				},
				complete: {
					title: t('point_flow.onlineOrders.confirmCompleteTitle'),
					desc: t('point_flow.onlineOrders.confirmCompleteDesc').replace(
						'{orderNo}',
						order.order_no
					),
					confirm: t('point_flow.onlineOrders.actionComplete')
				},
				'mark-paid': {
					title: t('point_flow.onlineOrders.confirmPaidTitle'),
					desc: t('point_flow.onlineOrders.confirmPaidDesc').replace('{orderNo}', order.order_no),
					confirm: t('point_flow.onlineOrders.actionMarkPaid')
				}
			} as const
		)[action]
	);

	$effect(() => {
		if (!open) return;
		errorMessage = null;
	});

	async function handleConfirm() {
		if (isSubmitting) return;
		isSubmitting = true;
		errorMessage = null;

		try {
			const run = {
				pack: () => packOnlineOrder(order.id),
				ready: () => readyOnlineOrder(order.id),
				deliver: () => deliverOnlineOrder(order.id),
				complete: () => completeOnlineOrder(order.id),
				'mark-paid': () => markOnlineOrderPaid(order.id)
			}[action];
			const detail = await run();
			await onDone?.(detail);
			toast.success(t('point_flow.onlineOrders.updatedToast'), {
				description: order.order_no
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
		return t('point_flow.onlineOrders.actionFailed');
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="sm:max-w-md">
		<Dialog.Header>
			<Dialog.Title>{content.title}</Dialog.Title>
			<Dialog.Description>{content.desc}</Dialog.Description>
		</Dialog.Header>

		{#if errorMessage}
			<ErrorAlert title={t('point_flow.onlineOrders.actionFailed')} message={errorMessage} />
		{/if}

		<Dialog.Footer>
			<Dialog.Close
				type="button"
				class={cn(buttonVariants({ variant: 'outline' }))}
				disabled={isSubmitting}
			>
				{t('common.keepOrder')}
			</Dialog.Close>
			<Button type="button" onclick={handleConfirm} disabled={isSubmitting}>
				{#if isSubmitting}
					<Spinner />
				{/if}
				{isSubmitting ? t('common.ordering') : content.confirm}
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
