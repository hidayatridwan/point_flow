<script lang="ts">
	import EyeIcon from '@lucide/svelte/icons/eye';
	import PencilIcon from '@lucide/svelte/icons/pencil';
	import {
		FlexRender,
		createSvelteTable,
		renderComponent
	} from '$lib/components/ui/data-table/index.js';
	import * as Table from '$lib/components/ui/table/index.js';
	import type { ProfitSharingPeriod } from '$lib/api/profit-sharing.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import { formatDate, formatIdr } from '$lib/formatters.js';
	import { createColumnHelper, getCoreRowModel } from '@tanstack/table-core';
	import PeriodStatusBadge from './period-status-badge.svelte';
	import { t } from '$lib/i18n/locale.svelte.js';

	let {
		periods,
		emptyMessage = t('finances.profitSharing.emptyDefault'),
		isLoading = false,
		onEdit
	}: {
		periods: ProfitSharingPeriod[];
		emptyMessage?: string;
		isLoading?: boolean;
		onEdit?: (period: ProfitSharingPeriod) => void;
	} = $props();

	const skeletonRows = [...Array(10).keys()];
	const columnHelper = createColumnHelper<ProfitSharingPeriod>();

	const columns = [
		columnHelper.accessor('period_name', {
			header: () => t('finances.profitSharing.colName'),
			cell: (info) => info.getValue()
		}),
		columnHelper.display({
			id: 'range',
			header: () => t('finances.profitSharing.colRange'),
			cell: (info) =>
				`${formatDate(info.row.original.start_date)} – ${formatDate(info.row.original.end_date)}`
		}),
		columnHelper.display({
			id: 'pool',
			header: () => t('finances.profitSharing.colPool'),
			cell: (info) => {
				const pool = info.row.original.total_pool;
				return pool === null || pool === undefined ? '—' : formatIdr(pool);
			}
		}),
		columnHelper.accessor('status', {
			header: () => t('finances.profitSharing.colStatus'),
			cell: (info) => renderComponent(PeriodStatusBadge, { status: info.getValue() })
		}),
		columnHelper.display({
			id: 'actions',
			header: () => t('finances.profitSharing.colAction'),
			cell: () => ''
		})
	];

	const table = createSvelteTable({
		get data() {
			return periods;
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
									<Skeleton class={column.id === 'actions' ? 'mx-auto h-8 w-8' : 'h-5 w-32'} />
								</Table.Cell>
							{/each}
						</Table.Row>
					{/each}
				{:else if table.getRowModel().rows.length > 0}
					{#each table.getRowModel().rows as row (row.id)}
						<Table.Row>
							{#each row.getVisibleCells() as cell (cell.id)}
								{#if cell.column.id === 'actions'}
									<Table.Cell class="w-20 text-center">
										<div class="flex items-center justify-center gap-1">
											<Button
												variant="ghost"
												size="icon-sm"
												class="size-8"
												aria-label={t('common.detail')}
												title={t('common.detail')}
												href={`/profit-sharing/${cell.row.original.id}`}
											>
												<EyeIcon />
											</Button>
											{#if cell.row.original.status === 'draft'}
												<Button
													type="button"
													variant="ghost"
													size="icon-sm"
													class="size-8"
													aria-label={t('common.edit')}
													title={t('common.edit')}
													onclick={() => onEdit?.(cell.row.original)}
												>
													<PencilIcon />
												</Button>
											{/if}
										</div>
									</Table.Cell>
								{:else}
									<Table.Cell>
										<FlexRender content={cell.column.columnDef.cell} context={cell.getContext()} />
									</Table.Cell>
								{/if}
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
