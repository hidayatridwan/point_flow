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
	import { t } from '$lib/i18n/locale.svelte.js';

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
		return t('procurement.goodsReceipts.detail.loadFailed');
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="max-h-[90vh] overflow-y-auto sm:max-w-4xl">
		<Dialog.Header>
			<Dialog.Title>{t('procurement.goodsReceipts.detail.title')}</Dialog.Title>
			<Dialog.Description>{t('procurement.goodsReceipts.detail.desc')}</Dialog.Description>
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
		{:else if goodsReceipt}
			<div class="grid gap-5">
				<div class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
					<DetailField label={t('procurement.goodsReceipts.colGrNo')} value={goodsReceipt.gr_no} />
					<DetailField label={t('procurement.goodsReceipts.poNo')} value={goodsReceipt.po_no} />
					<DetailField label={t('common.status')}>
						<GoodsReceiptStatusBadge status={goodsReceipt.status} />
					</DetailField>
					<DetailField
						label={t('procurement.goodsReceipts.receivedDate')}
						value={formatDateTime(goodsReceipt.received_at)}
					/>
					<DetailField
						label={t('common.postedAt')}
						value={formatDateTime(goodsReceipt.posted_at)}
					/>
					<DetailField
						label={t('common.cancelledAt')}
						value={formatDateTime(goodsReceipt.cancelled_at)}
					/>
				</div>

				<div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
					<DetailField label={t('common.notes')} value={goodsReceipt.notes || '-'} />
					<DetailField
						label={t('procurement.goodsReceipts.cancel.cancelReason')}
						value={goodsReceipt.cancel_reason || '-'}
					/>
					<DetailField
						label={t('common.createdAt')}
						value={formatDateTime(goodsReceipt.created_at)}
					/>
					<DetailField
						label={t('common.updatedAt')}
						value={formatDateTime(goodsReceipt.updated_at)}
					/>
				</div>

				<div class="overflow-hidden rounded-md border">
					<div class="overflow-x-auto">
						<Table.Root>
							<Table.Header>
								<Table.Row>
									<Table.Head>{t('procurement.goodsReceipts.detail.colProduct')}</Table.Head>
									<Table.Head>{t('procurement.goodsReceipts.detail.colSku')}</Table.Head>
									<Table.Head>{t('procurement.goodsReceipts.detail.colUnit')}</Table.Head>
									<Table.Head>{t('procurement.goodsReceipts.detail.colReceived')}</Table.Head>
									<Table.Head>{t('procurement.goodsReceipts.detail.colUnitCost')}</Table.Head>
									<Table.Head>{t('procurement.goodsReceipts.detail.colLineTotal')}</Table.Head>
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
