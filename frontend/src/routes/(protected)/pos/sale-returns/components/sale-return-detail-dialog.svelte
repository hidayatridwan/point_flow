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
	import { t } from '$lib/i18n/locale.svelte.js';

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
		return t('pos.saleReturns.detail.loadFailed');
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
			<ErrorAlert title={t('common.loadFailed')} message={errorMessage} />
		{:else if saleReturn}
			<div class="grid gap-5">
				<div class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
					<DetailField label={t('pos.saleReturns.detail.returnNo')} value={saleReturn.return_no} />
					<DetailField label={t('pos.saleReturns.detail.status')}>
						<SaleReturnStatusBadge status={saleReturn.status} />
					</DetailField>
					<DetailField label={t('pos.saleReturns.detail.returnedAt')} value={formatDateTime(saleReturn.returned_at)} />
					<DetailField label={t('pos.saleReturns.detail.saleTx')} value={saleReturn.sale_transaction_no} />
					<DetailField label={t('pos.saleReturns.detail.cashier')} value={saleReturn.cashier_name} />
					<DetailField label={t('pos.saleReturns.detail.register')} value={saleReturn.cash_register_code || '-'} />
					<DetailField label={t('pos.saleReturns.detail.subtotal')} value={formatIdr(saleReturn.subtotal_amount)} />
					<DetailField label={t('pos.saleReturns.detail.refund')} value={formatIdr(saleReturn.refund_amount)} />
					<DetailField label={t('pos.saleReturns.detail.cancelledAt')} value={formatDateTime(saleReturn.cancelled_at)} />
					<DetailField label={t('pos.saleReturns.detail.updatedAt')} value={formatDateTime(saleReturn.updated_at)} />
				</div>

				<div class="grid grid-cols-1 gap-3 sm:grid-cols-3">
					<DetailField label={t('pos.saleReturns.detail.returnReason')} value={saleReturn.reason || '-'} />
					<DetailField label={t('pos.saleReturns.detail.notes')} value={saleReturn.notes || '-'} />
					<DetailField label={t('pos.saleReturns.detail.cancelReason')} value={saleReturn.cancel_reason || '-'} />
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
										<Table.Cell>{item.restock ? t('pos.saleReturns.detail.yesLabel') : t('pos.saleReturns.detail.noLabel')}</Table.Cell>
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
