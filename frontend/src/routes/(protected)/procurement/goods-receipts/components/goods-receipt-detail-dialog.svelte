<script lang="ts">
	import { getGoodsReceipt, type GoodsReceiptDetail } from '$lib/api/goods-receipts.js';
	import { ApiError } from '$lib/api/types.js';
	import DetailField from '$lib/components/detail-field.svelte';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import * as Table from '$lib/components/ui/table/index.js';
	import { formatDateTime, formatIdr, formatNumber } from '$lib/formatters.js';
	import GoodsReceiptStatusBadge from './goods-receipt-status-badge.svelte';

	let {
		goodsReceiptId,
		open = $bindable(false)
	}: {
		goodsReceiptId: number;
		open: boolean;
	} = $props();

	let goodsReceipt = $state<GoodsReceiptDetail | null>(null);
	let isLoading = $state(false);
	let errorMessage = $state<string | null>(null);
	let latestRequestId = 0;

	$effect(() => {
		if (!open) return;
		void loadGoodsReceiptDetail(goodsReceiptId);
	});

	async function loadGoodsReceiptDetail(id: number) {
		const requestId = ++latestRequestId;
		isLoading = true;
		errorMessage = null;

		try {
			const result = await getGoodsReceipt(id);
			if (requestId !== latestRequestId) return;
			goodsReceipt = result;
		} catch (error) {
			if (requestId !== latestRequestId) return;
			goodsReceipt = null;
			errorMessage = getErrorMessage(error);
		} finally {
			if (requestId === latestRequestId) {
				isLoading = false;
			}
		}
	}

	function getErrorMessage(error: unknown) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return 'Failed to load goods receipt details.';
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="max-h-[90vh] overflow-y-auto sm:max-w-4xl">
		<Dialog.Header>
			<Dialog.Title>Goods Receipt Details</Dialog.Title>
			<Dialog.Description>Purchase order, status, and received line items.</Dialog.Description>
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
		{:else if goodsReceipt}
			<div class="grid gap-5">
				<div class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
					<DetailField label="GR No" value={goodsReceipt.gr_no} />
					<DetailField label="PO No" value={goodsReceipt.po_no} />
					<DetailField label="Status">
						<GoodsReceiptStatusBadge status={goodsReceipt.status} />
					</DetailField>
					<DetailField label="Received At" value={formatDateTime(goodsReceipt.received_at)} />
					<DetailField label="Posted At" value={formatDateTime(goodsReceipt.posted_at)} />
					<DetailField label="Cancelled At" value={formatDateTime(goodsReceipt.cancelled_at)} />
				</div>

				<div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
					<DetailField label="Notes" value={goodsReceipt.notes || '-'} />
					<DetailField label="Cancel Reason" value={goodsReceipt.cancel_reason || '-'} />
					<DetailField label="Created At" value={formatDateTime(goodsReceipt.created_at)} />
					<DetailField label="Updated At" value={formatDateTime(goodsReceipt.updated_at)} />
				</div>

				<div class="overflow-hidden rounded-md border">
					<div class="overflow-x-auto">
						<Table.Root>
							<Table.Header>
								<Table.Row>
									<Table.Head>Product</Table.Head>
									<Table.Head>SKU</Table.Head>
									<Table.Head>Unit</Table.Head>
									<Table.Head>Received</Table.Head>
									<Table.Head>Unit Cost</Table.Head>
									<Table.Head>Line Total</Table.Head>
								</Table.Row>
							</Table.Header>
							<Table.Body>
								{#each goodsReceipt.items as item (item.id)}
									<Table.Row>
										<Table.Cell>{item.product_name}</Table.Cell>
										<Table.Cell>{item.product_sku}</Table.Cell>
										<Table.Cell>{item.unit}</Table.Cell>
										<Table.Cell>{formatNumber(item.quantity_received)}</Table.Cell>
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
