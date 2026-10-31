<script lang="ts">
	import {
		FlexRender,
		createSvelteTable,
		renderComponent
	} from '$lib/components/ui/data-table/index.js';
	import type { GoodsReceiptReport } from '$lib/api/reports.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import * as Table from '$lib/components/ui/table/index.js';
	import { formatDateTime, formatIdr, formatNumber } from '$lib/formatters.js';
	import { createColumnHelper, getCoreRowModel } from '@tanstack/table-core';
	import { t } from '$lib/i18n/locale.svelte.js';
	import GoodsReceiptStatusBadge from '../../../procurement/goods-receipts/components/goods-receipt-status-badge.svelte';

	let {
		reports,
		emptyMessage = 'No goods receipt report rows.',
		isLoading = false
	}: {
		reports: GoodsReceiptReport[];
		emptyMessage?: string;
		isLoading?: boolean;
	} = $props();

	const skeletonRows = [...Array(10).keys()];
	const columnHelper = createColumnHelper<GoodsReceiptReport>();

	const columns = [
		columnHelper.accessor('gr_no', {
			header: () => t('procurement.goodsReceipts.colGrNo'),
			cell: (info) => info.getValue()
		}),
		columnHelper.accessor('po_no', {
			header: () => t('procurement.goodsReceipts.colPoNo'),
			cell: (info) => info.getValue()
		}),
		columnHelper.accessor('supplier_name', {
			header: () => t('procurement.goodsReceipts.colSupplier'),
			cell: (info) => info.getValue()
		}),
		columnHelper.accessor('received_at', {
			header: () => t('procurement.goodsReceipts.colReceivedDate'),
			cell: (info) => formatDateTime(info.getValue())
		}),
		columnHelper.accessor('status', {
			header: () => t('procurement.goodsReceipts.colStatus'),
			cell: (info) => renderComponent(GoodsReceiptStatusBadge, { status: info.getValue() })
		}),
		columnHelper.accessor('item_count', {
			header: () => t('common.colItems'),
			cell: (info) => formatNumber(info.getValue())
		}),
		columnHelper.accessor('total_quantity_received', {
			header: () => t('common.colReceivedQty'),
			cell: (info) => formatNumber(info.getValue())
		}),
		columnHelper.accessor('total_amount', {
			header: () => t('procurement.goodsReceipts.colTotal'),
			cell: (info) => formatIdr(info.getValue())
		}),
		columnHelper.accessor('updated_at', {
			header: () => t('common.updatedAt'),
			cell: (info) => formatDateTime(info.getValue())
		})
	];

	const table = createSvelteTable({
		get data() {
			return reports;
		},
		columns,
		getCoreRowModel: getCoreRowModel()
	});

	function getSkeletonClass(columnIndex: number) {
		const widths = [
			'h-5 w-36',
			'h-5 w-36',
			'h-5 w-44',
			'h-5 w-32',
			'h-5 w-24',
			'h-5 w-16',
			'h-5 w-24',
			'h-5 w-32',
			'h-5 w-32'
		];
		return widths[columnIndex] ?? 'h-5 w-full';
	}
</script>

<div class="overflow-hidden rounded-md border">
	<div class="overflow-x-auto">
		<Table.Root>
			<Table.Header>
				{#each table.getHeaderGroups() as headerGroup (headerGroup.id)}
					<Table.Row>
						{#each headerGroup.headers as header (header.id)}
							<Table.Head>
								{#if !header.isPlaceholder}
									<FlexRender
										content={header.column.columnDef.header}
										context={header.getContext()}
									/>
								{/if}
							</Table.Head>
						{/each}
					</Table.Row>
				{/each}
			</Table.Header>
			<Table.Body>
				{#if isLoading}
					{#each skeletonRows as rowIndex (rowIndex)}
						<Table.Row>
							{#each columns as column, columnIndex (column.id ?? columnIndex)}
								<Table.Cell>
									<Skeleton class={getSkeletonClass(columnIndex)} />
								</Table.Cell>
							{/each}
						</Table.Row>
					{/each}
				{:else if table.getRowModel().rows.length > 0}
					{#each table.getRowModel().rows as row (row.id)}
						<Table.Row>
							{#each row.getVisibleCells() as cell (cell.id)}
								<Table.Cell>
									<FlexRender content={cell.column.columnDef.cell} context={cell.getContext()} />
								</Table.Cell>
							{/each}
						</Table.Row>
					{/each}
				{:else}
					<Table.Row>
						<Table.Cell colspan={columns.length} class="h-24 text-center text-muted-foreground">
							{emptyMessage}
						</Table.Cell>
					</Table.Row>
				{/if}
			</Table.Body>
		</Table.Root>
	</div>
</div>
