<script lang="ts">
	import {
		FlexRender,
		createSvelteTable,
		renderComponent
	} from '$lib/components/ui/data-table/index.js';
	import type { AuditLog } from '$lib/api/audit-logs.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import * as Table from '$lib/components/ui/table/index.js';
	import { formatDateTime } from '$lib/formatters.js';
	import { createColumnHelper, getCoreRowModel } from '@tanstack/table-core';
	import AuditLogActions from './audit-log-actions.svelte';
	import AuditLogActorCell from './audit-log-actor-cell.svelte';
	import AuditLogOutcomeBadge from './audit-log-outcome-badge.svelte';
	import AuditLogRequestIdCell from './audit-log-request-id-cell.svelte';
	import AuditLogResourceCell from './audit-log-resource-cell.svelte';

	let {
		auditLogs,
		emptyMessage = 'No audit logs.',
		isLoading = false
	}: {
		auditLogs: AuditLog[];
		emptyMessage?: string;
		isLoading?: boolean;
	} = $props();

	const skeletonRows = [...Array(10).keys()];
	const columnHelper = createColumnHelper<AuditLog>();

	const columns = [
		columnHelper.accessor('occurred_at', {
			header: 'Occurred At',
			cell: (info) => formatDateTime(info.getValue())
		}),
		columnHelper.accessor('outcome', {
			header: 'Outcome',
			cell: (info) => renderComponent(AuditLogOutcomeBadge, { outcome: info.getValue() })
		}),
		columnHelper.accessor('action', {
			header: 'Action',
			cell: (info) => info.getValue()
		}),
		columnHelper.display({
			id: 'resource',
			header: () => 'Resource',
			cell: (info) =>
				renderComponent(AuditLogResourceCell, {
					resourceType: info.row.original.resource_type,
					resourceId: info.row.original.resource_id,
					resourceLabel: info.row.original.resource_label
				})
		}),
		columnHelper.display({
			id: 'actor',
			header: () => 'Actor',
			cell: (info) =>
				renderComponent(AuditLogActorCell, {
					name: info.row.original.actor_name,
					email: info.row.original.actor_email,
					actorId: info.row.original.actor_id
				})
		}),
		columnHelper.accessor('request_id', {
			header: 'Request ID',
			cell: (info) => renderComponent(AuditLogRequestIdCell, { requestId: info.getValue() })
		}),
		columnHelper.display({
			id: 'actions',
			header: () => 'Action',
			cell: (info) => renderComponent(AuditLogActions, { auditLog: info.row.original })
		})
	];

	const table = createSvelteTable({
		get data() {
			return auditLogs;
		},
		columns,
		getCoreRowModel: getCoreRowModel()
	});

	function getSkeletonClass(columnIndex: number) {
		const widths = ['h-5 w-36', 'h-5 w-24', 'h-5 w-40', 'h-5 w-36', 'h-5 w-44', 'h-5 w-44'];
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
