<script lang="ts">
	import BanIcon from '@lucide/svelte/icons/ban';
	import EllipsisIcon from '@lucide/svelte/icons/ellipsis';
	import EyeIcon from '@lucide/svelte/icons/eye';
	import SendIcon from '@lucide/svelte/icons/send';
	import type { PurchaseOrder } from '$lib/api/purchase-orders.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js';
	import { t } from '$lib/i18n/locale.svelte.js';
	import PurchaseOrderCancelDialog from './purchase-order-cancel-dialog.svelte';
	import PurchaseOrderDetailDialog from './purchase-order-detail-dialog.svelte';
	import PurchaseOrderOrderAlertDialog from './purchase-order-order-alert-dialog.svelte';

	let {
		purchaseOrder,
		onChanged
	}: {
		purchaseOrder: PurchaseOrder;
		onChanged?: (purchaseOrder: PurchaseOrder) => void | Promise<void>;
	} = $props();

	let isDetailOpen = $state(false);
	let isOrderOpen = $state(false);
	let isCancelOpen = $state(false);

	const canOrder = $derived(purchaseOrder.status === 'draft');
	const canCancel = $derived(
		purchaseOrder.status === 'draft' ||
			purchaseOrder.status === 'ordered' ||
			purchaseOrder.status === 'partially_received'
	);

	function openDetail() {
		isDetailOpen = true;
	}

	function openOrder() {
		isOrderOpen = true;
	}

	function openCancel() {
		isCancelOpen = true;
	}
</script>

<div class="flex justify-center">
	<DropdownMenu.Root>
		<DropdownMenu.Trigger>
			{#snippet child({ props })}
				<Button
					{...props}
					variant="ghost"
					size="icon-sm"
					class="size-8"
					aria-label={t('common.openActions').replace('{docNo}', purchaseOrder.po_no)}
					title={t('common.action')}
				>
					<EllipsisIcon />
				</Button>
			{/snippet}
		</DropdownMenu.Trigger>
		<DropdownMenu.Content align="end" class="min-w-44">
			<DropdownMenu.Item onSelect={openDetail}>
				<EyeIcon />
				{t('common.detail')}
			</DropdownMenu.Item>
			{#if canOrder}
				<DropdownMenu.Item onSelect={openOrder}>
					<SendIcon />
					{t('procurement.purchaseOrders.order.orderBtn')}
				</DropdownMenu.Item>
			{/if}
			{#if canCancel}
				<DropdownMenu.Item variant="destructive" onSelect={openCancel}>
					<BanIcon />
					{t('procurement.purchaseOrders.cancelOrderBtn')}
				</DropdownMenu.Item>
			{/if}
		</DropdownMenu.Content>
	</DropdownMenu.Root>

	<PurchaseOrderDetailDialog purchaseOrderId={purchaseOrder.id} bind:open={isDetailOpen} />
	<PurchaseOrderOrderAlertDialog {purchaseOrder} bind:open={isOrderOpen} onOrdered={onChanged} />
	<PurchaseOrderCancelDialog {purchaseOrder} bind:open={isCancelOpen} onCancelled={onChanged} />
</div>
