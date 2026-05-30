<script lang="ts">
	import { getSale, type SaleDetail } from '$lib/api/sales.js';
	import { ApiError } from '$lib/api/types.js';
	import DetailField from '$lib/components/detail-field.svelte';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import * as Table from '$lib/components/ui/table/index.js';
	import { formatDateTime, formatIdr, formatNumber } from '$lib/formatters.js';
	import SaleStatusBadge from './sale-status-badge.svelte';
	import { t } from '$lib/i18n/locale.svelte.js';

	let {
		saleId,
		open = $bindable(false)
	}: {
		saleId: number;
		open: boolean;
	} = $props();

	let sale = $state<SaleDetail | null>(null);
	let isLoading = $state(false);
	let errorMessage = $state<string | null>(null);
	let latestRequestId = 0;

	$effect(() => {
		if (!open) return;
		void loadSaleDetail(saleId);
	});

	async function loadSaleDetail(id: number) {
		const requestId = ++latestRequestId;
		isLoading = true;
		errorMessage = null;

		try {
			const result = await getSale(id);
			if (requestId !== latestRequestId) return;
			sale = result;
		} catch (error) {
			if (requestId !== latestRequestId) return;
			sale = null;
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
		return t('pos.sales.detail.loadFailed');
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="max-h-[90vh] overflow-y-auto sm:max-w-4xl">
		<Dialog.Header>
			<Dialog.Title>Sale Details</Dialog.Title>
			<Dialog.Description>Transaction payment and line item information.</Dialog.Description>
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
		{:else if sale}
			<div class="grid gap-5">
				<div class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
					<DetailField label={t('pos.sales.detail.transactionNo')} value={sale.transaction_no} />
					<DetailField label={t('pos.sales.detail.status')}>
						<SaleStatusBadge status={sale.status} />
					</DetailField>
					<DetailField label={t('pos.sales.detail.soldAt')} value={formatDateTime(sale.sold_at)} />
					<DetailField label={t('pos.sales.detail.cashier')} value={sale.cashier_name} />
					<DetailField label={t('pos.sales.detail.register')} value={sale.cash_register_code || '-'} />
					<DetailField label={t('pos.sales.detail.total')} value={formatIdr(sale.total_amount)} />
					<DetailField label={t('pos.sales.detail.paid')} value={formatIdr(sale.paid_amount)} />
					<DetailField label={t('pos.sales.detail.change')} value={formatIdr(sale.change_amount)} />
					<DetailField label={t('pos.sales.detail.cancelledAt')} value={formatDateTime(sale.cancelled_at)} />
					<DetailField label={t('pos.sales.detail.updatedAt')} value={formatDateTime(sale.updated_at)} />
				</div>

				<div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
					<DetailField label={t('pos.sales.detail.notes')} value={sale.notes || '-'} />
					<DetailField label={t('pos.sales.detail.cancelReason')} value={sale.cancel_reason || '-'} />
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
								</Table.Row>
							</Table.Header>
							<Table.Body>
								{#each sale.items as item (item.id)}
									<Table.Row>
										<Table.Cell>{item.product_name}</Table.Cell>
										<Table.Cell>{item.product_sku}</Table.Cell>
										<Table.Cell>{item.unit}</Table.Cell>
										<Table.Cell>{formatNumber(item.quantity)}</Table.Cell>
										<Table.Cell>{formatIdr(item.unit_price)}</Table.Cell>
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
