<script lang="ts">
	import {
		FlexRender,
		createSvelteTable,
		renderComponent
	} from '$lib/components/ui/data-table/index.js';
	import type { StockReportItem } from '$lib/api/stock-report.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import * as Table from '$lib/components/ui/table/index.js';
	import { formatIdr, formatNumber } from '$lib/formatters.js';
	import { createColumnHelper, getCoreRowModel } from '@tanstack/table-core';
	import StockReportStatusBadge from './stock-report-status-badge.svelte';

	let {
		items,
		emptyMessage = 'No stock report items.',
		isLoading = false
	}: {
		items: StockReportItem[];
		emptyMessage?: string;
		isLoading?: boolean;
	} = $props();

	const skeletonRows = [...Array(10).keys()];
	const columnHelper = createColumnHelper<StockReportItem>();

	const columns = [
		columnHelper.accessor('product_name', {
			header: 'Product',
			cell: (info) => info.getValue()
		}),
		columnHelper.accessor('sku', {
			header: 'SKU',
			cell: (info) => info.getValue()
		}),
		columnHelper.accessor('barcode', {
			header: 'Barcode',
			cell: (info) => info.getValue() ?? '-'
		}),
		columnHelper.accessor('category_name', {
			header: 'Category',
			cell: (info) => info.getValue() ?? '-'
		}),
		columnHelper.accessor('stock', {
			header: 'Current Stock',
			cell: (info) => `${formatNumber(info.getValue())} ${info.row.original.unit}`
		}),
		columnHelper.accessor('minimum_stock', {
			header: 'Minimum Stock',
			cell: (info) => `${formatNumber(info.getValue())} ${info.row.original.unit}`
		}),
		columnHelper.accessor('quantity_on_order', {
			header: 'On Order',
			cell: (info) => `${formatNumber(info.getValue())} ${info.row.original.unit}`
		}),
		columnHelper.accessor('selling_price', {
			header: 'Selling Price',
			cell: (info) => formatIdr(info.getValue())
		}),
		columnHelper.display({
			id: 'status',
			header: 'Status',
			cell: (info) => renderComponent(StockReportStatusBadge, { item: info.row.original })
		})
	];

	const table = createSvelteTable({
		get data() {
			return items;
		},
		columns,
		getCoreRowModel: getCoreRowModel()
	});

	function getSkeletonClass(columnIndex: number) {
		const widths = [
			'h-5 w-44',
			'h-5 w-32',
			'h-5 w-32',
			'h-5 w-32',
			'h-5 w-28',
			'h-5 w-28',
			'h-5 w-24',
			'h-5 w-28',
			'h-5 w-24'
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
