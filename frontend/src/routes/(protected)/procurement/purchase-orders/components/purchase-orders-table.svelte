<script lang="ts">
	import {
		FlexRender,
		createSvelteTable,
		renderComponent
	} from '$lib/components/ui/data-table/index.js';
	import type { PurchaseOrder } from '$lib/api/purchase-orders.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import * as Table from '$lib/components/ui/table/index.js';
	import { formatDate, formatDateTime, formatIdr } from '$lib/formatters.js';
	import { createColumnHelper, getCoreRowModel } from '@tanstack/table-core';
	import PurchaseOrderActions from './purchase-order-actions.svelte';
	import PurchaseOrderStatusBadge from './purchase-order-status-badge.svelte';

	let {
		purchaseOrders,
		emptyMessage = 'No purchase orders.',
		isLoading = false,
		onPurchaseOrderChanged
	}: {
		purchaseOrders: PurchaseOrder[];
		emptyMessage?: string;
		isLoading?: boolean;
		onPurchaseOrderChanged?: (purchaseOrder: PurchaseOrder) => void | Promise<void>;
	} = $props();

	const skeletonRows = [...Array(10).keys()];
	const columnHelper = createColumnHelper<PurchaseOrder>();

	const columns = [
		columnHelper.accessor('po_no', {
			header: 'PO No',
			cell: (info) => info.getValue()
		}),
		columnHelper.accessor('supplier_name', {
			header: 'Supplier',
			cell: (info) => info.getValue()
		}),
		columnHelper.accessor('order_date', {
			header: 'Order Date',
			cell: (info) => formatDate(info.getValue())
		}),
		columnHelper.accessor('expected_date', {
			header: 'Expected Date',
			cell: (info) => formatDate(info.getValue())
		}),
		columnHelper.accessor('status', {
			header: 'Status',
			cell: (info) => renderComponent(PurchaseOrderStatusBadge, { status: info.getValue() })
		}),
		columnHelper.accessor('subtotal_amount', {
			header: 'Subtotal',
			cell: (info) => formatIdr(info.getValue())
		}),
		columnHelper.accessor('updated_at', {
			header: 'Updated At',
			cell: (info) => formatDateTime(info.getValue())
		}),
		columnHelper.display({
			id: 'actions',
			header: () => 'Action',
			cell: (info) =>
				renderComponent(PurchaseOrderActions, {
					purchaseOrder: info.row.original,
					onChanged: onPurchaseOrderChanged
				})
		})
	];

	const table = createSvelteTable({
		get data() {
			return purchaseOrders;
		},
		columns,
		getCoreRowModel: getCoreRowModel()
	});

	function getSkeletonClass(columnIndex: number) {
		const widths = [
			'h-5 w-36',
			'h-5 w-44',
			'h-5 w-28',
			'h-5 w-28',
			'h-5 w-28',
			'h-5 w-28',
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
