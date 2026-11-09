<script lang="ts">
	import {
		FlexRender,
		createSvelteTable,
		renderComponent
	} from '$lib/components/ui/data-table/index.js';
	import * as Table from '$lib/components/ui/table/index.js';
	import type { Member } from '$lib/api/members.js';
	import StatusBadge from '$lib/components/status-badge.svelte';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import { formatDateTime } from '$lib/formatters.js';
	import { createColumnHelper, getCoreRowModel } from '@tanstack/table-core';
	import MemberActions from './member-actions.svelte';
	import { t } from '$lib/i18n/locale.svelte.js';

	let {
		members,
		emptyMessage = t('master.members.emptyDefault'),
		isLoading = false,
		onMemberDeleted,
		onMemberUpdated
	}: {
		members: Member[];
		emptyMessage?: string;
		isLoading?: boolean;
		onMemberDeleted?: (memberId: number) => void | Promise<void>;
		onMemberUpdated?: (member: Member) => void;
	} = $props();

	const skeletonRows = [...Array(10).keys()];
	const columnHelper = createColumnHelper<Member>();

	const columns = [
		columnHelper.accessor('member_no', {
			header: () => t('master.members.colMemberNo'),
			cell: (info) => info.getValue()
		}),
		columnHelper.accessor('full_name', {
			header: () => t('master.members.colName'),
			cell: (info) => info.getValue()
		}),
		columnHelper.accessor('phone', {
			header: () => t('master.members.colPhone'),
			cell: (info) => info.getValue() || '-'
		}),
		columnHelper.accessor('is_active', {
			header: () => t('master.members.colStatus'),
			cell: (info) => renderComponent(StatusBadge, { active: info.getValue() })
		}),
		columnHelper.accessor('joined_at', {
			header: () => t('master.members.colJoinedAt'),
			cell: (info) => formatDateTime(info.getValue())
		}),
		columnHelper.display({
			id: 'actions',
			header: () => t('master.members.colAction'),
			cell: (info) =>
				renderComponent(MemberActions, {
					member: info.row.original,
					onDeleted: onMemberDeleted,
					onUpdated: onMemberUpdated
				})
		})
	];

	const table = createSvelteTable({
		get data() {
			return members;
		},
		columns,
		getCoreRowModel: getCoreRowModel()
	});

	function getSkeletonClass(columnIndex: number) {
		const widths = ['h-5 w-32', 'h-5 w-44', 'h-5 w-32', 'h-5 w-20', 'h-5 w-36'];
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
