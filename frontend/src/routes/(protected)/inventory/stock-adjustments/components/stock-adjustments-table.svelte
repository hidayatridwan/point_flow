<script lang="ts">
	import {
		FlexRender,
		createSvelteTable,
		renderComponent
	} from '$lib/components/ui/data-table/index.js';
	import type { StockAdjustment } from '$lib/api/stock-adjustments.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import * as Table from '$lib/components/ui/table/index.js';
	import { formatDateTime } from '$lib/formatters.js';
	import { createColumnHelper, getCoreRowModel } from '@tanstack/table-core';
	import StockAdjustmentActions from './stock-adjustment-actions.svelte';
	import StockAdjustmentStatusBadge from './stock-adjustment-status-badge.svelte';

	let {
		stockAdjustments,
		emptyMessage = 'No stock adjustments.',
		isLoading = false,
		onStockAdjustmentChanged
	}: {
		stockAdjustments: StockAdjustment[];
		emptyMessage?: string;
		isLoading?: boolean;
		onStockAdjustmentChanged?: (stockAdjustment: StockAdjustment) => void | Promise<void>;
	} = $props();

	const skeletonRows = [...Array(10).keys()];
	const columnHelper = createColumnHelper<StockAdjustment>();

	const columns = [
		columnHelper.accessor('adjustment_no', {
			header: 'Adjustment No',
			cell: (info) => info.getValue()
		}),
		columnHelper.accessor('adjusted_at', {
			header: 'Adjusted At',
			cell: (info) => formatDateTime(info.getValue())
		}),
		columnHelper.accessor('status', {
			header: 'Status',
			cell: (info) => renderComponent(StockAdjustmentStatusBadge, { status: info.getValue() })
		}),
		columnHelper.accessor('reason', {
			header: 'Reason',
			cell: (info) => info.getValue()
		}),
		columnHelper.accessor('posted_at', {
			header: 'Posted At',
			cell: (info) => formatDateTime(info.getValue())
		}),
		columnHelper.accessor('updated_at', {
			header: 'Updated At',
			cell: (info) => formatDateTime(info.getValue())
		}),
		columnHelper.display({
			id: 'actions',
			header: () => 'Action',
			cell: (info) =>
				renderComponent(StockAdjustmentActions, {
					stockAdjustment: info.row.original,
					onChanged: onStockAdjustmentChanged
				})
		})
	];

	const table = createSvelteTable({
		get data() {
			return stockAdjustments;
		},
		columns,
		getCoreRowModel: getCoreRowModel()
	});

	function getSkeletonClass(columnIndex: number) {
		const widths = ['h-5 w-40', 'h-5 w-36', 'h-5 w-24', 'h-5 w-52', 'h-5 w-36', 'h-5 w-36'];
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
							<Table.Head class={header.column.id === 'actions' ? 'w-20 text-center' : undefined}>
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
								<Table.Cell class={column.id === 'actions' ? 'w-20 text-center' : undefined}>
									<Skeleton
										class={column.id === 'actions'
											? 'mx-auto h-8 w-8'
											: getSkeletonClass(columnIndex)}
									/>
								</Table.Cell>
							{/each}
						</Table.Row>
					{/each}
				{:else if table.getRowModel().rows.length > 0}
					{#each table.getRowModel().rows as row (row.id)}
						<Table.Row>
							{#each row.getVisibleCells() as cell (cell.id)}
								<Table.Cell class={cell.column.id === 'actions' ? 'w-20 text-center' : undefined}>
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
