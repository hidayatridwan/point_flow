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
		return 'Failed to load purchase order details.';
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="max-h-[90vh] overflow-y-auto sm:max-w-4xl">
		<Dialog.Header>
			<Dialog.Title>Purchase Order Details</Dialog.Title>
			<Dialog.Description>Supplier, status, and line item information.</Dialog.Description>
		</Dialog.Header>

		{#if isLoading}
			<div class="grid gap-3">
				<Skeleton class="h-5 w-48" />
				<Skeleton class="h-5 w-36" />
				<Skeleton class="h-5 w-52" />
				<Skeleton class="h-32 w-full" />
			</div>
		{:else if errorMessage}
			<ErrorAlert title="Failed to load details." message={errorMessage} />
		{:else if purchaseOrder}
			<div class="grid gap-5">
				<div class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
					<DetailField label="PO No" value={purchaseOrder.po_no} />
					<DetailField label="Supplier" value={purchaseOrder.supplier_name} />
					<DetailField label="Status">
						<PurchaseOrderStatusBadge status={purchaseOrder.status} />
					</DetailField>
					<DetailField label="Order Date" value={formatDate(purchaseOrder.order_date)} />
					<DetailField label="Expected Date" value={formatDate(purchaseOrder.expected_date)} />
					<DetailField label="Subtotal" value={formatIdr(purchaseOrder.subtotal_amount)} />
				</div>

				<div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
					<DetailField label="Notes" value={purchaseOrder.notes || '-'} />
					<DetailField label="Cancel Reason" value={purchaseOrder.cancel_reason || '-'} />
					<DetailField label="Ordered At" value={formatDateTime(purchaseOrder.ordered_at)} />
					<DetailField label="Cancelled At" value={formatDateTime(purchaseOrder.cancelled_at)} />
					<DetailField label="Created At" value={formatDateTime(purchaseOrder.created_at)} />
					<DetailField label="Updated At" value={formatDateTime(purchaseOrder.updated_at)} />
				</div>

				<div class="overflow-hidden rounded-md border">
					<div class="overflow-x-auto">
						<Table.Root>
							<Table.Header>
								<Table.Row>
									<Table.Head>Product</Table.Head>
									<Table.Head>SKU</Table.Head>
									<Table.Head>Unit</Table.Head>
									<Table.Head>Ordered</Table.Head>
									<Table.Head>Received</Table.Head>
									<Table.Head>Remaining</Table.Head>
									<Table.Head>Unit Cost</Table.Head>
									<Table.Head>Line Total</Table.Head>
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
