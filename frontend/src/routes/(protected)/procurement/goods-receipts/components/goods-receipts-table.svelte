<script lang="ts">
	import {
		FlexRender,
		createSvelteTable,
		renderComponent
	} from '$lib/components/ui/data-table/index.js';
	import type { GoodsReceipt } from '$lib/api/goods-receipts.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import * as Table from '$lib/components/ui/table/index.js';
	import { formatDateTime } from '$lib/formatters.js';
	import { createColumnHelper, getCoreRowModel } from '@tanstack/table-core';
	import { t } from '$lib/i18n/locale.svelte.js';
	import GoodsReceiptActions from './goods-receipt-actions.svelte';
	import GoodsReceiptStatusBadge from './goods-receipt-status-badge.svelte';

	let {
		goodsReceipts,
		emptyMessage = 'No goods receipts.',
		isLoading = false,
		onGoodsReceiptChanged
	}: {
		goodsReceipts: GoodsReceipt[];
		emptyMessage?: string;
		isLoading?: boolean;
		onGoodsReceiptChanged?: (goodsReceipt: GoodsReceipt) => void | Promise<void>;
	} = $props();

	const skeletonRows = [...Array(10).keys()];
	const columnHelper = createColumnHelper<GoodsReceipt>();

	const columns = [
		columnHelper.accessor('gr_no', {
			header: () => t('procurement.goodsReceipts.colGrNo'),
			cell: (info) => info.getValue()
		}),
		columnHelper.accessor('po_no', {
			header: () => t('procurement.goodsReceipts.colPoNo'),
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
		columnHelper.accessor('posted_at', {
			header: () => t('common.postedAt'),
			cell: (info) => formatDateTime(info.getValue())
		}),
		columnHelper.accessor('updated_at', {
			header: () => t('common.updatedAt'),
			cell: (info) => formatDateTime(info.getValue())
		}),
		columnHelper.display({
			id: 'actions',
			header: () => t('procurement.goodsReceipts.colAction'),
			cell: (info) =>
				renderComponent(GoodsReceiptActions, {
					goodsReceipt: info.row.original,
					onChanged: onGoodsReceiptChanged
				})
		})
	];

	const table = createSvelteTable({
		get data() {
			return goodsReceipts;
		},
		columns,
		getCoreRowModel: getCoreRowModel()
	});

	function getSkeletonClass(columnIndex: number) {
		const widths = ['h-5 w-36', 'h-5 w-36', 'h-5 w-36', 'h-5 w-24', 'h-5 w-36', 'h-5 w-36'];
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
