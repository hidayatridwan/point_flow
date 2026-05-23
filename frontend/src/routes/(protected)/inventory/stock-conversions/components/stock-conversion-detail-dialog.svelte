<script lang="ts">
	import { getStockConversion, type StockConversionDetail } from '$lib/api/stock-conversions.js';
	import { ApiError } from '$lib/api/types.js';
	import DetailField from '$lib/components/detail-field.svelte';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import * as Table from '$lib/components/ui/table/index.js';
	import { formatDateTime, formatNumber } from '$lib/formatters.js';
	import StockConversionStatusBadge from './stock-conversion-status-badge.svelte';

	let {
		stockConversionId,
		open = $bindable(false)
	}: {
		stockConversionId: number;
		open: boolean;
	} = $props();

	let stockConversion = $state<StockConversionDetail | null>(null);
	let isLoading = $state(false);
	let errorMessage = $state<string | null>(null);
	let latestRequestId = 0;

	$effect(() => {
		if (!open) return;
		void loadStockConversionDetail(stockConversionId);
	});

	async function loadStockConversionDetail(id: number) {
		const requestId = ++latestRequestId;
		isLoading = true;
		errorMessage = null;

		try {
			const result = await getStockConversion(id);
			if (requestId !== latestRequestId) return;
			stockConversion = result;
		} catch (error) {
			if (requestId !== latestRequestId) return;
			stockConversion = null;
			errorMessage = getErrorMessage(error);
		} finally {
			if (requestId === latestRequestId) {
				isLoading = false;
			}
		}
	}

	function formatStockChange(before: number, after: number, unit: string) {
		return `${formatNumber(before)} ${unit} -> ${formatNumber(after)} ${unit}`;
	}

	function getErrorMessage(error: unknown) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return 'Failed to load stock conversion details.';
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="max-h-[90vh] overflow-y-auto sm:max-w-5xl">
		<Dialog.Header>
			<Dialog.Title>Stock Conversion Details</Dialog.Title>
			<Dialog.Description>Conversion status and source to target SKU line items.</Dialog.Description
			>
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
		{:else if stockConversion}
			<div class="grid gap-5">
				<div class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
					<DetailField label="Conversion No" value={stockConversion.conversion_no} />
					<DetailField label="Status">
						<StockConversionStatusBadge status={stockConversion.status} />
					</DetailField>
					<DetailField label="Converted At" value={formatDateTime(stockConversion.converted_at)} />
					<DetailField label="Posted At" value={formatDateTime(stockConversion.posted_at)} />
					<DetailField label="Cancelled At" value={formatDateTime(stockConversion.cancelled_at)} />
					<DetailField label="Updated At" value={formatDateTime(stockConversion.updated_at)} />
				</div>

				<div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
					<DetailField label="Notes" value={stockConversion.notes || '-'} />
					<DetailField label="Cancel Reason" value={stockConversion.cancel_reason || '-'} />
					<DetailField label="Created At" value={formatDateTime(stockConversion.created_at)} />
				</div>

				<div class="overflow-hidden rounded-md border">
					<div class="overflow-x-auto">
						<Table.Root>
							<Table.Header>
								<Table.Row>
									<Table.Head>From Product</Table.Head>
									<Table.Head>From SKU</Table.Head>
									<Table.Head>Out Qty</Table.Head>
									<Table.Head>From Stock</Table.Head>
									<Table.Head>To Product</Table.Head>
									<Table.Head>To SKU</Table.Head>
									<Table.Head>In Qty</Table.Head>
									<Table.Head>To Stock</Table.Head>
									<Table.Head>Notes</Table.Head>
								</Table.Row>
							</Table.Header>
							<Table.Body>
								{#each stockConversion.items as item (item.id)}
									<Table.Row>
										<Table.Cell>{item.from_product_name}</Table.Cell>
										<Table.Cell>{item.from_product_sku}</Table.Cell>
										<Table.Cell>{formatNumber(item.from_quantity)} {item.from_unit}</Table.Cell>
										<Table.Cell>
											{formatStockChange(
												item.from_stock_before,
												item.from_stock_after,
												item.from_unit
											)}
										</Table.Cell>
										<Table.Cell>{item.to_product_name}</Table.Cell>
										<Table.Cell>{item.to_product_sku}</Table.Cell>
										<Table.Cell>{formatNumber(item.to_quantity)} {item.to_unit}</Table.Cell>
										<Table.Cell>
											{formatStockChange(item.to_stock_before, item.to_stock_after, item.to_unit)}
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
