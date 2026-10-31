<script lang="ts">
	import {
		getPurchaseOrder,
		type PurchaseOrderDetail,
		type PurchaseOrderItem
	} from '$lib/api/purchase-orders.js';
	import { ApiError } from '$lib/api/types.js';
	import DetailField from '$lib/components/detail-field.svelte';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import * as Table from '$lib/components/ui/table/index.js';
	import { formatDate, formatDateTime, formatIdr, formatNumber } from '$lib/formatters.js';
	import PurchaseOrderStatusBadge from './purchase-order-status-badge.svelte';
	import { t } from '$lib/i18n/locale.svelte.js';

	let {
		purchaseOrderId,
		open = $bindable(false)
	}: {
		purchaseOrderId: number;
		open: boolean;
	} = $props();

	let purchaseOrder = $state<PurchaseOrderDetail | null>(null);
	let isLoading = $state(false);
	let errorMessage = $state<string | null>(null);
	let latestRequestId = 0;

	$effect(() => {
		if (!open) return;
		void loadPurchaseOrderDetail(purchaseOrderId);
	});

	async function loadPurchaseOrderDetail(id: number) {
		const requestId = ++latestRequestId;
		isLoading = true;
		errorMessage = null;

		try {
			const result = await getPurchaseOrder(id);
			if (requestId !== latestRequestId) return;
			purchaseOrder = result;
		} catch (error) {
			if (requestId !== latestRequestId) return;
			purchaseOrder = null;
			errorMessage = getErrorMessage(error);
		} finally {
			if (requestId === latestRequestId) {
				isLoading = false;
			}
		}
	}

	function getRemainingQuantity(item: PurchaseOrderItem) {
		return item.quantity_ordered - item.quantity_received;
	}

	function getErrorMessage(error: unknown) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return t('procurement.purchaseOrders.detail.loadFailed');
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="max-h-[90vh] overflow-y-auto sm:max-w-4xl">
		<Dialog.Header>
			<Dialog.Title>{t('procurement.purchaseOrders.detail.title')}</Dialog.Title>
			<Dialog.Description>{t('procurement.purchaseOrders.detail.desc')}</Dialog.Description>
		</Dialog.Header>

		{#if isLoading}
			<div class="grid gap-3">
				<Skeleton class="h-5 w-48" />
				<Skeleton class="h-5 w-36" />
				<Skeleton class="h-5 w-52" />
				<Skeleton class="h-32 w-full" />
			</div>
		{:else if errorMessage}
			<ErrorAlert title={t('common.loadFailed')} message={errorMessage} />
		{:else if purchaseOrder}
			<div class="grid gap-5">
				<div class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
					<DetailField
						label={t('procurement.purchaseOrders.colPoNo')}
						value={purchaseOrder.po_no}
					/>
					<DetailField
						label={t('procurement.purchaseOrders.supplier')}
						value={purchaseOrder.supplier_name}
					/>
					<DetailField label={t('common.status')}>
						<PurchaseOrderStatusBadge status={purchaseOrder.status} />
					</DetailField>
					<DetailField
						label={t('procurement.purchaseOrders.orderDate')}
						value={formatDate(purchaseOrder.order_date)}
					/>
					<DetailField
						label={t('procurement.purchaseOrders.expectedDate')}
						value={formatDate(purchaseOrder.expected_date)}
					/>
					<DetailField
						label={t('procurement.purchaseOrders.subtotal')}
						value={formatIdr(purchaseOrder.subtotal_amount)}
					/>
				</div>

				<div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
					<DetailField label={t('common.notes')} value={purchaseOrder.notes || '-'} />
					<DetailField
						label={t('procurement.purchaseOrders.cancelReason')}
						value={purchaseOrder.cancel_reason || '-'}
					/>
					<DetailField
						label={t('procurement.purchaseOrders.detail.orderedAt')}
						value={formatDateTime(purchaseOrder.ordered_at)}
					/>
					<DetailField
						label={t('common.cancelledAt')}
						value={formatDateTime(purchaseOrder.cancelled_at)}
					/>
					<DetailField
						label={t('common.createdAt')}
						value={formatDateTime(purchaseOrder.created_at)}
					/>
					<DetailField
						label={t('common.updatedAt')}
						value={formatDateTime(purchaseOrder.updated_at)}
					/>
				</div>

				<div class="overflow-hidden rounded-md border">
					<div class="overflow-x-auto">
						<Table.Root>
							<Table.Header>
								<Table.Row>
									<Table.Head>{t('procurement.purchaseOrders.detail.colProduct')}</Table.Head>
									<Table.Head>{t('procurement.purchaseOrders.detail.colSku')}</Table.Head>
									<Table.Head>{t('procurement.purchaseOrders.detail.colUnit')}</Table.Head>
									<Table.Head>{t('procurement.purchaseOrders.detail.colOrdered')}</Table.Head>
									<Table.Head>{t('procurement.purchaseOrders.detail.colReceived')}</Table.Head>
									<Table.Head>{t('procurement.purchaseOrders.detail.colRemaining')}</Table.Head>
									<Table.Head>{t('procurement.purchaseOrders.detail.colUnitCost')}</Table.Head>
									<Table.Head>{t('procurement.purchaseOrders.detail.colLineTotal')}</Table.Head>
								</Table.Row>
							</Table.Header>
							<Table.Body>
								{#each purchaseOrder.items as item (item.id)}
									<Table.Row>
										<Table.Cell>{item.product_name}</Table.Cell>
										<Table.Cell>{item.product_sku}</Table.Cell>
										<Table.Cell>{item.unit}</Table.Cell>
										<Table.Cell>{formatNumber(item.quantity_ordered)}</Table.Cell>
										<Table.Cell>{formatNumber(item.quantity_received)}</Table.Cell>
										<Table.Cell>{formatNumber(getRemainingQuantity(item))}</Table.Cell>
										<Table.Cell>{formatIdr(item.unit_cost)}</Table.Cell>
										<Table.Cell>{formatIdr(item.line_total)}</Table.Cell>
									</Table.Row>
								{/each}
							</Table.Body>
						</Table.Root>
					</div>
				</div>
			</div>
		{/if}
	</Dialog.Content>
</Dialog.Root>
