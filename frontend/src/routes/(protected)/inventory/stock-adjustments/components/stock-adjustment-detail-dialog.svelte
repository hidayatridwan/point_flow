<script lang="ts">
	import { getStockAdjustment, type StockAdjustmentDetail } from '$lib/api/stock-adjustments.js';
	import { ApiError } from '$lib/api/types.js';
	import DetailField from '$lib/components/detail-field.svelte';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import * as Table from '$lib/components/ui/table/index.js';
	import { formatDateTime, formatNumber } from '$lib/formatters.js';
	import StockAdjustmentStatusBadge from './stock-adjustment-status-badge.svelte';
	import { t } from '$lib/i18n/locale.svelte.js';

	let {
		stockAdjustmentId,
		open = $bindable(false)
	}: {
		stockAdjustmentId: number;
		open: boolean;
	} = $props();

	let stockAdjustment = $state<StockAdjustmentDetail | null>(null);
	let isLoading = $state(false);
	let errorMessage = $state<string | null>(null);
	let latestRequestId = 0;

	$effect(() => {
		if (!open) return;
		void loadStockAdjustmentDetail(stockAdjustmentId);
	});

	async function loadStockAdjustmentDetail(id: number) {
		const requestId = ++latestRequestId;
		isLoading = true;
		errorMessage = null;

		try {
			const result = await getStockAdjustment(id);
			if (requestId !== latestRequestId) return;
			stockAdjustment = result;
		} catch (error) {
			if (requestId !== latestRequestId) return;
			stockAdjustment = null;
			errorMessage = getErrorMessage(error);
		} finally {
			if (requestId === latestRequestId) {
				isLoading = false;
			}
		}
	}

	function formatSignedNumber(value: number) {
		const formattedValue = formatNumber(Math.abs(value));
		if (value > 0) return `+${formattedValue}`;
		if (value < 0) return `-${formattedValue}`;
		return formattedValue;
	}

	function getDeltaClass(value: number) {
		if (value > 0) return 'text-emerald-600 dark:text-emerald-400';
		if (value < 0) return 'text-rose-600 dark:text-rose-400';
		return 'text-muted-foreground';
	}

	function getErrorMessage(error: unknown) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return t('inventory.stockAdjustments.detail.loadFailed');
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="max-h-[90vh] overflow-y-auto sm:max-w-4xl">
		<Dialog.Header>
			<Dialog.Title>{t('inventory.stockAdjustments.detail.title')}</Dialog.Title>
			<Dialog.Description>{t('inventory.stockAdjustments.detail.desc')}</Dialog.Description>
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
		{:else if stockAdjustment}
			<div class="grid gap-5">
				<div class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
					<DetailField
						label={t('inventory.stockAdjustments.colAdjustmentNo')}
						value={stockAdjustment.adjustment_no}
					/>
					<DetailField label={t('common.status')}>
						<StockAdjustmentStatusBadge status={stockAdjustment.status} />
					</DetailField>
					<DetailField
						label={t('inventory.stockAdjustments.adjustedAt')}
						value={formatDateTime(stockAdjustment.adjusted_at)}
					/>
					<DetailField
						label={t('common.postedAt')}
						value={formatDateTime(stockAdjustment.posted_at)}
					/>
					<DetailField
						label={t('common.cancelledAt')}
						value={formatDateTime(stockAdjustment.cancelled_at)}
					/>
					<DetailField
						label={t('common.updatedAt')}
						value={formatDateTime(stockAdjustment.updated_at)}
					/>
				</div>

				<div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
					<DetailField
						label={t('inventory.stockAdjustments.reason')}
						value={stockAdjustment.reason}
					/>
					<DetailField
						label={t('inventory.stockAdjustments.detail.colNotes')}
						value={stockAdjustment.notes || '-'}
					/>
					<DetailField
						label={t('inventory.stockAdjustments.cancel.cancelReason')}
						value={stockAdjustment.cancel_reason || '-'}
					/>
					<DetailField
						label={t('common.createdAt')}
						value={formatDateTime(stockAdjustment.created_at)}
					/>
				</div>

				<div class="overflow-hidden rounded-md border">
					<div class="overflow-x-auto">
						<Table.Root>
							<Table.Header>
								<Table.Row>
									<Table.Head>{t('inventory.stockAdjustments.detail.colProduct')}</Table.Head>
									<Table.Head>{t('inventory.stockAdjustments.detail.colSku')}</Table.Head>
									<Table.Head>{t('inventory.stockAdjustments.detail.colUnit')}</Table.Head>
									<Table.Head>{t('inventory.stockAdjustments.detail.colBefore')}</Table.Head>
									<Table.Head>{t('inventory.stockAdjustments.detail.colAfter')}</Table.Head>
									<Table.Head>{t('inventory.stockAdjustments.detail.colDelta')}</Table.Head>
									<Table.Head>{t('inventory.stockAdjustments.detail.colNotes')}</Table.Head>
								</Table.Row>
							</Table.Header>
							<Table.Body>
								{#each stockAdjustment.items as item (item.id)}
									<Table.Row>
										<Table.Cell>{item.product_name}</Table.Cell>
										<Table.Cell>{item.product_sku}</Table.Cell>
										<Table.Cell>{item.unit}</Table.Cell>
										<Table.Cell>{formatNumber(item.stock_before)}</Table.Cell>
										<Table.Cell>{formatNumber(item.stock_after)}</Table.Cell>
										<Table.Cell class={getDeltaClass(item.quantity_delta)}>
											{formatSignedNumber(item.quantity_delta)}
										</Table.Cell>
										<Table.Cell>{item.notes || '-'}</Table.Cell>
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
