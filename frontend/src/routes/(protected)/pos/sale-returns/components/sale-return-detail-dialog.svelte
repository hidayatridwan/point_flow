<script lang="ts">
	import { getSaleReturn, type SaleReturnDetail } from '$lib/api/sales.js';
	import { ApiError } from '$lib/api/types.js';
	import DetailField from '$lib/components/detail-field.svelte';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import * as Table from '$lib/components/ui/table/index.js';
	import { formatDateTime, formatIdr, formatNumber } from '$lib/formatters.js';
	import SaleReturnStatusBadge from './sale-return-status-badge.svelte';

	let {
		saleReturnId,
		open = $bindable(false)
	}: {
		saleReturnId: number;
		open: boolean;
	} = $props();

	let saleReturn = $state<SaleReturnDetail | null>(null);
	let isLoading = $state(false);
	let errorMessage = $state<string | null>(null);
	let latestRequestId = 0;

	$effect(() => {
		if (!open) return;
		void loadSaleReturnDetail(saleReturnId);
	});

	async function loadSaleReturnDetail(id: number) {
		const requestId = ++latestRequestId;
		isLoading = true;
		errorMessage = null;

		try {
			const result = await getSaleReturn(id);
			if (requestId !== latestRequestId) return;
			saleReturn = result;
		} catch (error) {
			if (requestId !== latestRequestId) return;
			saleReturn = null;
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
		return 'Failed to load return details.';
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="max-h-[90vh] overflow-y-auto sm:max-w-4xl">
		<Dialog.Header>
			<Dialog.Title>Return Details</Dialog.Title>
			<Dialog.Description>Refund, register, and returned line item information.</Dialog.Description>
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
		{:else if saleReturn}
			<div class="grid gap-5">
				<div class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
					<DetailField label="Return No" value={saleReturn.return_no} />
					<DetailField label="Status">
						<SaleReturnStatusBadge status={saleReturn.status} />
					</DetailField>
					<DetailField label="Returned At" value={formatDateTime(saleReturn.returned_at)} />
					<DetailField label="Sale Transaction" value={saleReturn.sale_transaction_no} />
					<DetailField label="Cashier" value={saleReturn.cashier_name} />
					<DetailField label="Register" value={saleReturn.cash_register_code || '-'} />
					<DetailField label="Subtotal" value={formatIdr(saleReturn.subtotal_amount)} />
					<DetailField label="Refund" value={formatIdr(saleReturn.refund_amount)} />
					<DetailField label="Cancelled At" value={formatDateTime(saleReturn.cancelled_at)} />
					<DetailField label="Updated At" value={formatDateTime(saleReturn.updated_at)} />
				</div>

				<div class="grid grid-cols-1 gap-3 sm:grid-cols-3">
					<DetailField label="Return Reason" value={saleReturn.reason || '-'} />
					<DetailField label="Notes" value={saleReturn.notes || '-'} />
					<DetailField label="Cancel Reason" value={saleReturn.cancel_reason || '-'} />
				</div>

				<div class="overflow-hidden rounded-md border">
					<div class="overflow-x-auto">
						<Table.Root>
							<Table.Header>
								<Table.Row>
									<Table.Head>Product</Table.Head>
									<Table.Head>SKU</Table.Head>
									<Table.Head>Unit</Table.Head>
									<Table.Head>Qty</Table.Head>
									<Table.Head>Unit Price</Table.Head>
									<Table.Head>Line Total</Table.Head>
									<Table.Head>Restock</Table.Head>
									<Table.Head>Reason</Table.Head>
								</Table.Row>
							</Table.Header>
							<Table.Body>
								{#each saleReturn.items as item (item.id)}
									<Table.Row>
										<Table.Cell>{item.product_name}</Table.Cell>
										<Table.Cell>{item.product_sku}</Table.Cell>
										<Table.Cell>{item.unit}</Table.Cell>
										<Table.Cell>{formatNumber(item.quantity)}</Table.Cell>
										<Table.Cell>{formatIdr(item.unit_price)}</Table.Cell>
										<Table.Cell>{formatIdr(item.line_total)}</Table.Cell>
										<Table.Cell>{item.restock ? 'Yes' : 'No'}</Table.Cell>
										<Table.Cell>{item.reason || '-'}</Table.Cell>
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
