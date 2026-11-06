<script lang="ts">
	import {
		FlexRender,
		createSvelteTable,
		renderComponent
	} from '$lib/components/ui/data-table/index.js';
	import type { Sale } from '$lib/api/sales.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import * as Table from '$lib/components/ui/table/index.js';
	import { formatDateTime, formatIdr } from '$lib/formatters.js';
	import { createColumnHelper, getCoreRowModel } from '@tanstack/table-core';
	import SaleActions from './sale-actions.svelte';
	import SaleStatusBadge from './sale-status-badge.svelte';
	import { t } from '$lib/i18n/locale.svelte.js';

	let {
		sales,
		emptyMessage = 'No sales.',
		isLoading = false,
		onSaleChanged
	}: {
		sales: Sale[];
		emptyMessage?: string;
		isLoading?: boolean;
		onSaleChanged?: (sale: Sale) => void | Promise<void>;
	} = $props();

	const skeletonRows = [...Array(10).keys()];
	const columnHelper = createColumnHelper<Sale>();

	const columns = [
		columnHelper.accessor('transaction_no', {
			header: () => t('point_flow.sales.colTransactionNo'),
			cell: (info) => info.getValue()
		}),
		columnHelper.accessor('sold_at', {
			header: () => t('point_flow.sales.colSoldAt'),
			cell: (info) => formatDateTime(info.getValue())
		}),
		columnHelper.accessor('cashier_name', {
			header: () => t('point_flow.sales.colCashier'),
			cell: (info) => info.getValue()
		}),
		columnHelper.accessor('cash_register_code', {
			header: () => t('point_flow.sales.colRegister'),
			cell: (info) => info.getValue() ?? '-'
		}),
		columnHelper.accessor('status', {
			header: () => t('point_flow.sales.colStatus'),
			cell: (info) => renderComponent(SaleStatusBadge, { status: info.getValue() })
		}),
		columnHelper.accessor('total_amount', {
			header: () => t('point_flow.sales.colTotal'),
			cell: (info) => formatIdr(info.getValue())
		}),
		columnHelper.accessor('paid_amount', {
			header: () => t('point_flow.sales.colPaid'),
			cell: (info) => formatIdr(info.getValue())
		}),
		columnHelper.accessor('change_amount', {
			header: () => t('point_flow.sales.colChange'),
			cell: (info) => formatIdr(info.getValue())
		}),
		columnHelper.display({
			id: 'actions',
			header: () => t('point_flow.sales.colAction'),
			cell: (info) =>
				renderComponent(SaleActions, {
					sale: info.row.original,
					onChanged: onSaleChanged
				})
		})
	];

	const table = createSvelteTable({
		get data() {
			return sales;
		},
		columns,
		getCoreRowModel: getCoreRowModel()
	});

	function getSkeletonClass(columnIndex: number) {
		const widths = [
			'h-5 w-40',
			'h-5 w-36',
			'h-5 w-32',
			'h-5 w-24',
			'h-5 w-28',
			'h-5 w-28',
			'h-5 w-28',
			'h-5 w-28'
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
