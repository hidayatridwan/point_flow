<script lang="ts">
	import {
		FlexRender,
		createSvelteTable,
		renderComponent
	} from '$lib/components/ui/data-table/index.js';
	import * as Table from '$lib/components/ui/table/index.js';
	import type { ProductSku } from '$lib/api/product-skus.js';
	import StatusBadge from '$lib/components/status-badge.svelte';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import { formatDateTime, formatIdr, formatNumber } from '$lib/formatters.js';
	import { createColumnHelper, getCoreRowModel } from '@tanstack/table-core';
	import ProductSkuActions from './product-sku-actions.svelte';

	let {
		productSkus,
		emptyMessage = 'No product SKUs.',
		isLoading = false,
		onProductSkuDeleted,
		onProductSkuUpdated
	}: {
		productSkus: ProductSku[];
		emptyMessage?: string;
		isLoading?: boolean;
		onProductSkuDeleted?: (productSkuId: number) => void | Promise<void>;
		onProductSkuUpdated?: (productSku: ProductSku) => void;
	} = $props();

	const skeletonRows = [...Array(10).keys()];
	const columnHelper = createColumnHelper<ProductSku>();

	const columns = [
		columnHelper.accessor('product_name', {
			header: 'Product',
			cell: (info) => info.getValue()
		}),
		columnHelper.accessor('sku', {
			header: 'SKU',
			cell: (info) => info.getValue()
		}),
		columnHelper.accessor('unit_name', {
			header: 'Unit',
			cell: (info) => `${info.row.original.unit_name} (${info.row.original.unit_code})`
		}),
		columnHelper.accessor('selling_price', {
			header: 'Selling Price',
			cell: (info) => formatIdr(info.getValue())
		}),
		columnHelper.accessor('stock', {
			header: 'Stock',
			cell: (info) =>
				`${formatNumber(info.getValue())} / min ${formatNumber(info.row.original.minimum_stock)}`
		}),
		columnHelper.accessor('is_active', {
			header: 'Status',
			cell: (info) => renderComponent(StatusBadge, { active: info.getValue() })
		}),
		columnHelper.accessor('updated_at', {
			header: 'Updated At',
			cell: (info) => formatDateTime(info.getValue())
		}),
		columnHelper.display({
			id: 'actions',
			header: () => 'Action',
			cell: (info) =>
				renderComponent(ProductSkuActions, {
					productSku: info.row.original,
					onDeleted: onProductSkuDeleted,
					onUpdated: onProductSkuUpdated
				})
		})
	];

	const table = createSvelteTable({
		get data() {
			return productSkus;
		},
		columns,
		getCoreRowModel: getCoreRowModel()
	});

	function getSkeletonClass(columnIndex: number) {
		const widths = [
			'h-5 w-44',
			'h-5 w-32',
			'h-5 w-32',
			'h-5 w-28',
			'h-5 w-32',
			'h-5 w-20',
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
