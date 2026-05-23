<script lang="ts">
	import {
		FlexRender,
		createSvelteTable,
		renderComponent
	} from '$lib/components/ui/data-table/index.js';
	import type { PayrollSlip } from '$lib/api/payroll.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import * as Table from '$lib/components/ui/table/index.js';
	import { formatDate, formatDateTime, formatIdr } from '$lib/formatters.js';
	import { createColumnHelper, getCoreRowModel } from '@tanstack/table-core';
	import PayrollSlipActions from './payroll-slip-actions.svelte';

	let {
		payrollSlips,
		emptyMessage = 'No payroll slips.',
		isLoading = false
	}: {
		payrollSlips: PayrollSlip[];
		emptyMessage?: string;
		isLoading?: boolean;
	} = $props();

	const skeletonRows = [...Array(10).keys()];
	const columnHelper = createColumnHelper<PayrollSlip>();

	const columns = [
		columnHelper.accessor('slip_no', {
			header: 'Slip No',
			cell: (info) => info.getValue()
		}),
		columnHelper.accessor('employee_name', {
			header: 'Employee',
			cell: (info) => info.getValue()
		}),
		columnHelper.accessor('employee_position', {
			header: 'Position',
			cell: (info) => info.getValue()
		}),
		columnHelper.accessor('period_start', {
			header: 'Period Start',
			cell: (info) => formatDate(info.getValue())
		}),
		columnHelper.accessor('period_end', {
			header: 'Period End',
			cell: (info) => formatDate(info.getValue())
		}),
		columnHelper.accessor('net_salary', {
			header: 'Net Salary',
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
				renderComponent(PayrollSlipActions, {
					payrollSlip: info.row.original
				})
		})
	];

	const table = createSvelteTable({
		get data() {
			return payrollSlips;
		},
		columns,
		getCoreRowModel: getCoreRowModel()
	});

	function getSkeletonClass(columnIndex: number) {
		const widths = [
			'h-5 w-36',
			'h-5 w-44',
			'h-5 w-32',
			'h-5 w-28',
			'h-5 w-28',
			'h-5 w-32',
			'h-5 w-36'
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
