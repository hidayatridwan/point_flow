<script lang="ts">
	import {
		FlexRender,
		createSvelteTable,
		renderComponent
	} from '$lib/components/ui/data-table/index.js';
	import type { StockMovement } from '$lib/api/stock-movements.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import * as Table from '$lib/components/ui/table/index.js';
	import { formatDateTime, formatNumber } from '$lib/formatters.js';
	import { createColumnHelper, getCoreRowModel } from '@tanstack/table-core';
	import StockMovementDirectionBadge from './stock-movement-direction-badge.svelte';
	import StockMovementTypeBadge from './stock-movement-type-badge.svelte';

	let {
		stockMovements,
		emptyMessage = 'No stock movements.',
		isLoading = false
	}: {
		stockMovements: StockMovement[];
		emptyMessage?: string;
		isLoading?: boolean;
	} = $props();

	const skeletonRows = [...Array(10).keys()];
	const columnHelper = createColumnHelper<StockMovement>();

	const columns = [
		columnHelper.accessor('created_at', {
			header: 'Date',
			cell: (info) => formatDateTime(info.getValue())
		}),
		columnHelper.accessor('product_name', {
			header: 'Product',
			cell: (info) => info.getValue()
		}),
		columnHelper.accessor('sku', {
			header: 'SKU',
			cell: (info) => info.getValue()
		}),
		columnHelper.accessor('movement_type', {
			header: 'Type',
			cell: (info) => renderComponent(StockMovementTypeBadge, { type: info.getValue() })
		}),
		columnHelper.accessor('direction', {
			header: 'Direction',
			cell: (info) => renderComponent(StockMovementDirectionBadge, { direction: info.getValue() })
		}),
		columnHelper.accessor('quantity', {
			header: 'Quantity',
			cell: (info) => `${formatNumber(info.getValue())} ${info.row.original.unit}`
		}),
		columnHelper.accessor('stock_before', {
			header: 'Before',
			cell: (info) => `${formatNumber(info.getValue())} ${info.row.original.unit}`
		}),
		columnHelper.accessor('stock_after', {
			header: 'After',
			cell: (info) => `${formatNumber(info.getValue())} ${info.row.original.unit}`
		}),
		columnHelper.accessor('quantity_delta', {
			header: 'Delta',
			cell: (info) => formatSignedQuantity(info.getValue(), info.row.original.unit)
		}),
		columnHelper.display({
			id: 'reference',
			header: 'Reference',
			cell: (info) => getReferenceLabel(info.row.original)
		}),
		columnHelper.accessor('notes', {
			header: 'Notes',
			cell: (info) => info.getValue() ?? '-'
		})
	];

	const table = createSvelteTable({
		get data() {
			return stockMovements;
		},
		columns,
		getCoreRowModel: getCoreRowModel()
	});

	function formatSignedQuantity(value: number, unit: string) {
		const formattedValue = formatNumber(Math.abs(value));
		if (value > 0) return `+${formattedValue} ${unit}`;
		if (value < 0) return `-${formattedValue} ${unit}`;
		return `${formattedValue} ${unit}`;
	}

	function getReferenceLabel(stockMovement: StockMovement) {
		if (!stockMovement.reference_no) return '-';
		return `${getReferenceTypeLabel(stockMovement.reference_type)} ${stockMovement.reference_no}`;
	}

	function getReferenceTypeLabel(value: string) {
		const labels: Record<string, string> = {
			sale: 'Sale',
			goods_receipt: 'GR',
			stock_adjustment: 'Adjustment',
			stock_conversion: 'Conversion'
		};
		return labels[value] ?? 'Reference';
	}

	function getSkeletonClass(columnIndex: number) {
		const widths = [
			'h-5 w-36',
			'h-5 w-44',
			'h-5 w-32',
			'h-5 w-36',
			'h-5 w-20',
			'h-5 w-24',
			'h-5 w-24',
			'h-5 w-24',
			'h-5 w-24',
			'h-5 w-40',
			'h-5 w-48'
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
