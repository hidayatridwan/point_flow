<script lang="ts">
	import {
		FlexRender,
		createSvelteTable,
		renderComponent
	} from '$lib/components/ui/data-table/index.js';
	import type { CashDrawerMovement } from '$lib/api/cash-registers.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import * as Table from '$lib/components/ui/table/index.js';
	import { formatDateTime, formatIdr } from '$lib/formatters.js';
	import { createColumnHelper, getCoreRowModel } from '@tanstack/table-core';
	import { t } from '$lib/i18n/locale.svelte.js';
	import CashDrawerMovementActions from './cash-drawer-movement-actions.svelte';
	import CashDrawerMovementStatusBadge from './cash-drawer-movement-status-badge.svelte';
	import CashDrawerMovementTypeBadge from './cash-drawer-movement-type-badge.svelte';

	let {
		movements,
		emptyMessage = 'No cash drawer movements.',
		isLoading = false,
		onMovementChanged
	}: {
		movements: CashDrawerMovement[];
		emptyMessage?: string;
		isLoading?: boolean;
		onMovementChanged?: (movement: CashDrawerMovement) => void | Promise<void>;
	} = $props();

	const skeletonRows = [...Array(10).keys()];
	const columnHelper = createColumnHelper<CashDrawerMovement>();

	const columns = [
		columnHelper.accessor('movement_no', {
			header: () => t('common.movementNo'),
			cell: (info) => info.getValue()
		}),
		columnHelper.accessor('occurred_at', {
			header: () => t('common.occurredAt'),
			cell: (info) => formatDateTime(info.getValue())
		}),
		columnHelper.accessor('movement_type', {
			header: () => t('pos.cashMovements.filterType'),
			cell: (info) => renderComponent(CashDrawerMovementTypeBadge, { type: info.getValue() })
		}),
		columnHelper.accessor('direction', {
			header: () => t('common.direction'),
			cell: (info) =>
				info.getValue() === 'in'
					? t('inventory.stockMovements.directionIn')
					: t('inventory.stockMovements.directionOut')
		}),
		columnHelper.accessor('amount', {
			header: () => t('pos.cashMovements.amount'),
			cell: (info) => formatIdr(info.getValue())
		}),
		columnHelper.accessor('session_no', {
			header: () => t('pos.cashMovements.colSession'),
			cell: (info) => info.getValue()
		}),
		columnHelper.accessor('cash_register_code', {
			header: () => t('common.register'),
			cell: (info) => info.getValue()
		}),
		columnHelper.accessor('created_by_name', {
			header: () => t('common.createdBy'),
			cell: (info) => info.getValue()
		}),
		columnHelper.accessor('status', {
			header: () => t('common.status'),
			cell: (info) => renderComponent(CashDrawerMovementStatusBadge, { status: info.getValue() })
		}),
		columnHelper.display({
			id: 'actions',
			header: () => t('common.action'),
			cell: (info) =>
				renderComponent(CashDrawerMovementActions, {
					movement: info.row.original,
					onChanged: onMovementChanged
				})
		})
	];

	const table = createSvelteTable({
		get data() {
			return movements;
		},
		columns,
		getCoreRowModel: getCoreRowModel()
	});

	function getSkeletonClass(columnIndex: number) {
		const widths = [
			'h-5 w-40',
			'h-5 w-36',
			'h-5 w-28',
			'h-5 w-20',
			'h-5 w-28',
			'h-5 w-40',
			'h-5 w-24',
			'h-5 w-32',
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
