<script lang="ts">
	import {
		FlexRender,
		createSvelteTable,
		renderComponent
	} from '$lib/components/ui/data-table/index.js';
	import * as Table from '$lib/components/ui/table/index.js';
	import type { Role } from '$lib/api/roles.js';
	import type { User } from '$lib/api/users.js';
	import StatusBadge from '$lib/components/status-badge.svelte';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import { formatDateTime } from '$lib/formatters.js';
	import { createColumnHelper, getCoreRowModel } from '@tanstack/table-core';
	import UserActions from './user-actions.svelte';
	import UserPermissions from './user-permissions.svelte';
	import { t } from '$lib/i18n/locale.svelte.js';

	let {
		users,
		roles,
		emptyMessage = 'No users.',
		isLoading = false,
		onUserDeleted,
		onUserUpdated
	}: {
		users: User[];
		roles: Role[];
		emptyMessage?: string;
		isLoading?: boolean;
		onUserDeleted?: (userId: number) => void | Promise<void>;
		onUserUpdated?: (user: User) => void;
	} = $props();

	const skeletonRows = [...Array(10).keys()];
	const columnHelper = createColumnHelper<User>();

	const columns = [
		columnHelper.accessor('name', {
			header: () => t('config.users.colName'),
			cell: (info) => info.getValue()
		}),
		columnHelper.accessor('email', {
			header: () => t('config.users.colEmail'),
			cell: (info) => info.getValue()
		}),
		columnHelper.accessor('role_name', {
			header: () => t('config.users.colRole'),
			cell: (info) => info.getValue()
		}),
		columnHelper.accessor('permissions', {
			header: () => t('config.users.colPermissions'),
			cell: (info) => renderComponent(UserPermissions, { permissions: info.getValue() })
		}),
		columnHelper.accessor('is_active', {
			header: () => t('config.users.colStatus'),
			cell: (info) => renderComponent(StatusBadge, { active: info.getValue() })
		}),
		columnHelper.accessor('last_login_at', {
			header: () => t('config.users.colLastLogin'),
			cell: (info) => formatDateTime(info.getValue())
		}),
		columnHelper.accessor('updated_at', {
			header: () => t('config.users.colUpdatedAt'),
			cell: (info) => formatDateTime(info.getValue())
		}),
		columnHelper.display({
			id: 'actions',
			header: () => t('config.users.colAction'),
			cell: (info) =>
				renderComponent(UserActions, {
					user: info.row.original,
					roles,
					onDeleted: onUserDeleted,
					onUpdated: onUserUpdated
				})
		})
	];

	const table = createSvelteTable({
		get data() {
			return users;
		},
		columns,
		getCoreRowModel: getCoreRowModel()
	});

	function getSkeletonClass(columnIndex: number) {
		const widths = [
			'h-5 w-32',
			'h-5 w-48',
			'h-5 w-24',
			'h-5 w-56',
			'h-5 w-20',
			'h-5 w-32',
			'h-5 w-32'
		];
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
