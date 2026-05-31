<script lang="ts">
	import SendIcon from '@lucide/svelte/icons/send';
	import { orderPurchaseOrder, type PurchaseOrder } from '$lib/api/purchase-orders.js';
	import { ApiError } from '$lib/api/types.js';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import * as AlertDialog from '$lib/components/ui/alert-dialog/index.js';
	import { Spinner } from '$lib/components/ui/spinner/index.js';
	import { toast } from 'svelte-sonner';
	import { t } from '$lib/i18n/locale.svelte.js';

	let {
		purchaseOrder,
		open = $bindable(false),
		onOrdered
	}: {
		purchaseOrder: PurchaseOrder;
		open: boolean;
		onOrdered?: (purchaseOrder: PurchaseOrder) => void | Promise<void>;
	} = $props();

	let isSubmitting = $state(false);
	let errorMessage = $state<string | null>(null);

	$effect(() => {
		if (!open) return;
		errorMessage = null;
	});

	async function handleOrder(event: MouseEvent) {
		event.preventDefault();
		if (isSubmitting) return;

		isSubmitting = true;
		errorMessage = null;

		try {
			const orderedPurchaseOrder = await orderPurchaseOrder(purchaseOrder.id);
			await onOrdered?.(orderedPurchaseOrder);
			toast.success(t('procurement.purchaseOrders.order.orderedToast'), {
				description: `${orderedPurchaseOrder.po_no} is ready for goods receipt.`
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
		return t('procurement.purchaseOrders.order.orderFailed');
	}
</script>

<AlertDialog.Root bind:open>
	<AlertDialog.Content>
		<AlertDialog.Header>
			<AlertDialog.Title>Order Purchase Order?</AlertDialog.Title>
			<AlertDialog.Description>
				This action will mark <span class="font-medium text-foreground">{purchaseOrder.po_no}</span>
				as ordered. Ordered purchase orders can be used for goods receipts.
			</AlertDialog.Description>
		</AlertDialog.Header>

		{#if errorMessage}
			<ErrorAlert
				title={t('procurement.purchaseOrders.order.orderFailed')}
				message={errorMessage}
			/>
		{/if}

		<AlertDialog.Footer>
			<AlertDialog.Cancel disabled={isSubmitting}>Cancel</AlertDialog.Cancel>
			<AlertDialog.Action disabled={isSubmitting} onclick={handleOrder}>
				{#if isSubmitting}
					<Spinner />
				{:else}
					<SendIcon />
				{/if}
				{isSubmitting
					? t('procurement.purchaseOrders.order.ordering')
					: t('procurement.purchaseOrders.order.orderBtn')}
			</AlertDialog.Action>
		</AlertDialog.Footer>
	</AlertDialog.Content>
</AlertDialog.Root>
