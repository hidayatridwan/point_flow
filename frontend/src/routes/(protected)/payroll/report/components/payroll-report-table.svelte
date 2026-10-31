<script lang="ts">
	import { FlexRender, createSvelteTable } from '$lib/components/ui/data-table/index.js';
	import type { PayrollReport } from '$lib/api/payroll.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import * as Table from '$lib/components/ui/table/index.js';
	import { formatDate, formatIdr, formatNumber } from '$lib/formatters.js';
	import { createColumnHelper, getCoreRowModel } from '@tanstack/table-core';
	import { t } from '$lib/i18n/locale.svelte.js';

	let {
		reports,
		emptyMessage = 'No payroll report rows.',
		isLoading = false
	}: {
		reports: PayrollReport[];
		emptyMessage?: string;
		isLoading?: boolean;
	} = $props();

	const skeletonRows = [...Array(10).keys()];
	const columnHelper = createColumnHelper<PayrollReport>();

	const columns = [
		columnHelper.accessor('period_start', {
			header: () => t('payroll.slips.periodStart'),
			cell: (info) => formatDate(info.getValue())
		}),
		columnHelper.accessor('period_end', {
			header: () => t('payroll.slips.periodEnd'),
			cell: (info) => formatDate(info.getValue())
		}),
		columnHelper.accessor('slip_count', {
			header: () => t('payroll.report.colSlipCount'),
			cell: (info) => formatNumber(info.getValue())
		}),
		columnHelper.accessor('total_base_salary', {
			header: () => t('payroll.report.colTotalBaseSalary'),
			cell: (info) => formatIdr(info.getValue())
		}),
		columnHelper.accessor('total_bonus', {
			header: () => t('payroll.report.colTotalBonus'),
			cell: (info) => formatIdr(info.getValue())
		}),
		columnHelper.accessor('total_deduction', {
			header: () => t('payroll.report.colTotalDeduction'),
			cell: (info) => formatIdr(info.getValue())
		}),
		columnHelper.accessor('total_net_salary', {
			header: () => t('payroll.report.colTotalNetSalary'),
			cell: (info) => formatIdr(info.getValue())
		})
	];

	const table = createSvelteTable({
		get data() {
			return reports;
		},
		columns,
		getCoreRowModel: getCoreRowModel()
	});

	function getSkeletonClass(columnIndex: number) {
		const widths = [
			'h-5 w-28',
			'h-5 w-28',
			'h-5 w-20',
			'h-5 w-36',
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
							<Table.Head>
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
									<Skeleton class={getSkeletonClass(columnIndex)} />
								</Table.Cell>
							{/each}
						</Table.Row>
					{/each}
				{:else if table.getRowModel().rows.length > 0}
					{#each table.getRowModel().rows as row (row.id)}
						<Table.Row>
							{#each row.getVisibleCells() as cell (cell.id)}
								<Table.Cell>
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
