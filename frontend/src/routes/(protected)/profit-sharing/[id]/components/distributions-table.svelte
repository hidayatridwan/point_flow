<script lang="ts">
	import { createSvelteTable, FlexRender } from '$lib/components/ui/data-table/index.js';
	import * as Table from '$lib/components/ui/table/index.js';
	import type { ProfitSharingDistribution } from '$lib/api/profit-sharing.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import { formatIdr } from '$lib/formatters.js';
	import { createColumnHelper, getCoreRowModel } from '@tanstack/table-core';
	import { t } from '$lib/i18n/locale.svelte.js';

	let {
		distributions,
		emptyMessage = t('finances.profitSharing.distributionsEmpty'),
		isLoading = false
	}: {
		distributions: ProfitSharingDistribution[];
		emptyMessage?: string;
		isLoading?: boolean;
	} = $props();

	const skeletonRows = [...Array(8).keys()];
	const columnHelper = createColumnHelper<ProfitSharingDistribution>();

	const columns = [
		columnHelper.accessor('member_no', {
			header: () => t('finances.profitSharing.colMemberNo'),
			cell: (info) => info.getValue()
		}),
		columnHelper.accessor('full_name', {
			header: () => t('finances.profitSharing.colMemberName'),
			cell: (info) => `${info.getValue()} (${info.row.original.phone})`
		}),
		columnHelper.accessor('transaction_amount', {
			header: () => t('finances.profitSharing.colTransaction'),
			cell: (info) => formatIdr(info.getValue())
		}),
		columnHelper.accessor('share_pct', {
			header: () => t('finances.profitSharing.colSharePct'),
			cell: (info) => `${(info.getValue() * 100).toFixed(2)}%`
		}),
		columnHelper.accessor('share_amount', {
			header: () => t('finances.profitSharing.colShareAmount'),
			cell: (info) => formatIdr(info.getValue())
		})
	];

	const table = createSvelteTable({
		get data() {
			return distributions;
		},
		columns,
		getCoreRowModel: getCoreRowModel()
	});
</script>

<div class="overflow-hidden rounded-md border">
	<div class="overflow-x-auto">
		<Table.Root>
			<Table.Header>
				{#each table.getHeaderGroups() as headerGroup (headerGroup.id)}
					<Table.Row>
						{#each headerGroup.headers as header (header.id)}
							<Table.Head
								class={header.column.id === 'share_amount' ||
								header.column.id === 'transaction_amount'
									? 'text-right'
									: undefined}
							>
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
									<Skeleton class="h-5 w-32" />
								</Table.Cell>
							{/each}
						</Table.Row>
					{/each}
				{:else if table.getRowModel().rows.length > 0}
					{#each table.getRowModel().rows as row (row.id)}
						<Table.Row>
							{#each row.getVisibleCells() as cell (cell.id)}
								<Table.Cell
									class={cell.column.id === 'share_amount' ||
									cell.column.id === 'transaction_amount'
										? 'text-right tabular-nums'
										: undefined}
								>
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
