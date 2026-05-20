<script lang="ts">
	import {
		FlexRender,
		createSvelteTable,
		renderComponent
	} from '$lib/components/ui/data-table/index.js';
	import * as Table from '$lib/components/ui/table/index.js';
	import type { ProductCategory } from '$lib/api/product-categories.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import { createColumnHelper, getCoreRowModel } from '@tanstack/table-core';
	import ProductCategoryActions from './product-category-actions.svelte';
	import StatusBadge from './status-badge.svelte';

	let {
		categories,
		emptyMessage = 'Tidak ada kategori produk.',
		isLoading = false,
		onCategoryDeleted,
		onCategoryUpdated
	}: {
		categories: ProductCategory[];
		emptyMessage?: string;
		isLoading?: boolean;
		onCategoryDeleted?: (categoryId: number) => void | Promise<void>;
		onCategoryUpdated?: (category: ProductCategory) => void;
	} = $props();

	const skeletonRows = Array.from({ length: 10 });
	const columnHelper = createColumnHelper<ProductCategory>();
	const dateFormatter = new Intl.DateTimeFormat('id-ID', {
		dateStyle: 'medium',
		timeStyle: 'short'
	});

	const columns = [
		columnHelper.accessor('name', {
			header: 'Name',
			cell: (info) => info.getValue()
		}),
		columnHelper.accessor('description', {
			header: 'Description',
			cell: (info) => info.getValue() || '-'
		}),
		columnHelper.accessor('is_active', {
			header: 'Status',
			cell: (info) => renderComponent(StatusBadge, { active: info.getValue() })
		}),
		columnHelper.accessor('created_at', {
			header: 'Created At',
			cell: (info) => formatDate(info.getValue())
		}),
		columnHelper.accessor('updated_at', {
			header: 'Updated At',
			cell: (info) => formatDate(info.getValue())
		}),
		columnHelper.display({
			id: 'actions',
			header: () => 'Action',
			cell: (info) =>
				renderComponent(ProductCategoryActions, {
					category: info.row.original,
					onDeleted: onCategoryDeleted,
					onUpdated: onCategoryUpdated
				})
		})
	];

	const table = createSvelteTable({
		get data() {
			return categories;
		},
		columns,
		getCoreRowModel: getCoreRowModel()
	});

	function formatDate(value: string) {
		const date = new Date(value);
		if (Number.isNaN(date.getTime())) return '-';
		return dateFormatter.format(date);
	}

	function getSkeletonClass(columnIndex: number) {
		const widths = ['h-5 w-36', 'h-5 w-full max-w-md', 'h-5 w-20', 'h-5 w-32', 'h-5 w-32'];
		return widths[columnIndex] ?? 'h-5 w-full';
	}
</script>

<div class="rounded-md border">
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
				{#each skeletonRows as _, rowIndex (rowIndex)}
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
