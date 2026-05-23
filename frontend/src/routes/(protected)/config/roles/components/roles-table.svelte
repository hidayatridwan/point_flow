<script lang="ts">
	import {
		FlexRender,
		createSvelteTable,
		renderComponent
	} from '$lib/components/ui/data-table/index.js';
	import * as Table from '$lib/components/ui/table/index.js';
	import type { Permission, Role } from '$lib/api/roles.js';
	import StatusBadge from '$lib/components/status-badge.svelte';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import { formatDateTime } from '$lib/formatters.js';
	import { createColumnHelper, getCoreRowModel } from '@tanstack/table-core';
	import RoleActions from './role-actions.svelte';
	import RolePermissions from './role-permissions.svelte';

	let {
		roles,
		permissions,
		emptyMessage = 'No roles.',
		isLoading = false,
		onRoleDeleted,
		onRoleUpdated
	}: {
		roles: Role[];
		permissions: Permission[];
		emptyMessage?: string;
		isLoading?: boolean;
		onRoleDeleted?: (roleId: number) => void | Promise<void>;
		onRoleUpdated?: (role: Role) => void;
	} = $props();

	const skeletonRows = [...Array(10).keys()];
	const columnHelper = createColumnHelper<Role>();

	const columns = [
		columnHelper.accessor('code', {
			header: 'Code',
			cell: (info) => info.getValue()
		}),
		columnHelper.accessor('name', {
			header: 'Name',
			cell: (info) => info.getValue()
		}),
		columnHelper.accessor('permissions', {
			header: 'Permissions',
			cell: (info) => renderComponent(RolePermissions, { permissions: info.getValue() })
		}),
		columnHelper.accessor('is_system', {
			header: 'Type',
			cell: (info) =>
				renderComponent(StatusBadge, {
					active: info.getValue(),
					activeLabel: 'System',
					inactiveLabel: 'Custom'
				})
		}),
		columnHelper.accessor('updated_at', {
			header: 'Updated At',
			cell: (info) => formatDateTime(info.getValue())
		}),
		columnHelper.display({
			id: 'actions',
			header: () => 'Action',
			cell: (info) =>
				renderComponent(RoleActions, {
					role: info.row.original,
					permissions,
					onDeleted: onRoleDeleted,
					onUpdated: onRoleUpdated
				})
		})
	];

	const table = createSvelteTable({
		get data() {
			return roles;
		},
		columns,
		getCoreRowModel: getCoreRowModel()
	});

	function getSkeletonClass(columnIndex: number) {
		const widths = ['h-5 w-24', 'h-5 w-36', 'h-5 w-64', 'h-5 w-20', 'h-5 w-32'];
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
