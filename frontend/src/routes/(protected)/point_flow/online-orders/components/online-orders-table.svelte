<script lang="ts">
	import {
		FlexRender,
		createSvelteTable,
		renderComponent
	} from '$lib/components/ui/data-table/index.js';
	import type { OnlineOrder } from '$lib/api/online-orders.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import * as Table from '$lib/components/ui/table/index.js';
	import { formatIdr } from '$lib/formatters.js';
	import { createColumnHelper, getCoreRowModel } from '@tanstack/table-core';
	import OnlineOrderActions from './online-order-actions.svelte';
	import OnlineOrderStatusBadge from './online-order-status-badge.svelte';
	import { t } from '$lib/i18n/locale.svelte.js';

	let {
		orders,
		emptyMessage = 'No online orders.',
		isLoading = false,
		onOrderChanged
	}: {
		orders: OnlineOrder[];
		emptyMessage?: string;
		isLoading?: boolean;
		onOrderChanged?: (order: OnlineOrder) => void | Promise<void>;
	} = $props();

	const skeletonRows = [...Array(10).keys()];
	const columnHelper = createColumnHelper<OnlineOrder>();

	const columns = [
		columnHelper.accessor('order_no', {
			header: () => t('point_flow.onlineOrders.colOrderNo'),
			cell: (info) => info.getValue()
		}),
		columnHelper.accessor('customer_name', {
			header: () => t('point_flow.onlineOrders.colCustomer'),
			cell: (info) => `${info.getValue()} · ${info.row.original.customer_phone}`
		}),
		columnHelper.accessor('fulfillment_method', {
			header: () => t('point_flow.onlineOrders.colFulfillment'),
			cell: (info) =>
				info.getValue() === 'delivery'
					? t('point_flow.onlineOrders.fulfillmentDelivery')
					: t('point_flow.onlineOrders.fulfillmentPickup')
		}),
		columnHelper.accessor('payment_method', {
			header: () => t('point_flow.onlineOrders.colPayment'),
			cell: (info) => {
				const method =
					info.getValue() === 'transfer'
						? t('point_flow.onlineOrders.paymentTransfer')
						: info.getValue() === 'midtrans'
							? t('point_flow.onlineOrders.paymentMidtrans')
							: t('point_flow.onlineOrders.paymentPayAtStore');
				const paid =
					info.row.original.payment_status === 'paid'
						? t('point_flow.onlineOrders.paymentPaid')
						: t('point_flow.onlineOrders.paymentUnpaid');
				return `${method} · ${paid}`;
			}
		}),
		columnHelper.accessor('status', {
			header: () => t('point_flow.onlineOrders.colStatus'),
			cell: (info) => renderComponent(OnlineOrderStatusBadge, { status: info.getValue() })
		}),
		columnHelper.accessor('total_amount', {
			header: () => t('point_flow.onlineOrders.colTotal'),
			cell: (info) => formatIdr(info.getValue())
		}),
		columnHelper.display({
			id: 'actions',
			header: () => t('point_flow.onlineOrders.colAction'),
			cell: (info) =>
				renderComponent(OnlineOrderActions, {
					order: info.row.original,
					onChanged: () => onOrderChanged?.(info.row.original)
				})
		})
	];

	const table = createSvelteTable({
		get data() {
			return orders;
		},
		columns,
		getCoreRowModel: getCoreRowModel()
	});

	function getSkeletonClass(columnIndex: number) {
		const widths = [
			'h-5 w-36',
			'h-5 w-44',
			'h-5 w-24',
			'h-5 w-32',
			'h-5 w-24',
			'h-5 w-28',
			'h-5 w-32'
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
