<script lang="ts">
	import {
		FlexRender,
		createSvelteTable,
		renderComponent
	} from '$lib/components/ui/data-table/index.js';
	import * as Table from '$lib/components/ui/table/index.js';
	import type { Unit } from '$lib/api/units.js';
	import StatusBadge from '$lib/components/status-badge.svelte';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import { formatDateTime } from '$lib/formatters.js';
	import { createColumnHelper, getCoreRowModel } from '@tanstack/table-core';
	import UnitActions from './unit-actions.svelte';

	let {
		units,
		emptyMessage = 'No units.',
		isLoading = false,
		onUnitDeleted,
		onUnitUpdated
	}: {
		units: Unit[];
		emptyMessage?: string;
		isLoading?: boolean;
		onUnitDeleted?: (unitId: number) => void | Promise<void>;
		onUnitUpdated?: (unit: Unit) => void;
	} = $props();

	const skeletonRows = [...Array(10).keys()];
	const columnHelper = createColumnHelper<Unit>();

	const columns = [
		columnHelper.accessor('code', {
			header: 'Code',
			cell: (info) => info.getValue()
		}),
		columnHelper.accessor('name', {
			header: 'Name',
			cell: (info) => info.getValue()
		}),
		columnHelper.accessor('allow_decimal', {
			header: 'Decimal',
			cell: (info) =>
				renderComponent(StatusBadge, {
					active: info.getValue(),
					activeLabel: 'Allowed',
					inactiveLabel: 'Whole'
				})
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
				renderComponent(UnitActions, {
					unit: info.row.original,
					onDeleted: onUnitDeleted,
					onUpdated: onUnitUpdated
				})
		})
	];

	const table = createSvelteTable({
		get data() {
			return units;
		},
		columns,
		getCoreRowModel: getCoreRowModel()
	});

	function getSkeletonClass(columnIndex: number) {
		const widths = ['h-5 w-20', 'h-5 w-40', 'h-5 w-20', 'h-5 w-20', 'h-5 w-32'];
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
