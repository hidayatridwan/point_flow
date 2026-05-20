<script lang="ts" module>
	import favicon from '$lib/assets/favicon.svg';
	import BadgeDollarSignIcon from '@lucide/svelte/icons/badge-dollar-sign';
	import ChartColumnIcon from '@lucide/svelte/icons/chart-column';
	import DatabaseIcon from '@lucide/svelte/icons/database';
	import LayoutDashboardIcon from '@lucide/svelte/icons/layout-dashboard';
	import ShoppingCartIcon from '@lucide/svelte/icons/shopping-cart';
	import TruckIcon from '@lucide/svelte/icons/truck';
	import WarehouseIcon from '@lucide/svelte/icons/warehouse';

	const data = {
		user: {
			name: 'Admin',
			email: 'admin@example.com',
			avatar: favicon
		},
		navMain: [
			{
				title: 'Dashboard',
				url: '/dashboard',
				icon: LayoutDashboardIcon
			},
			{
				title: 'Kasir',
				url: '/pos/sales/new',
				icon: ShoppingCartIcon,
				items: [
					{
						title: 'Transaksi Baru',
						url: '/pos/sales/new'
					},
					{
						title: 'Riwayat Penjualan',
						url: '/pos/sales'
					}
				]
			},
			{
				title: 'Master Data',
				url: '/master/users',
				icon: DatabaseIcon,
				items: [
					{
						title: 'Users',
						url: '/master/users'
					},
					{
						title: 'Product Categories',
						url: '/master/product-categories'
					},
					{
						title: 'Products',
						url: '/master/products'
					},
					{
						title: 'Units',
						url: '/master/units'
					},
					{
						title: 'Product SKUs',
						url: '/master/product-skus'
					},
					{
						title: 'Suppliers',
						url: '/master/suppliers'
					},
					{
						title: 'Employees',
						url: '/master/employees'
					}
				]
			},
			{
				title: 'Procurement',
				url: '/procurement/purchase-orders',
				icon: TruckIcon,
				items: [
					{
						title: 'Purchase Orders',
						url: '/procurement/purchase-orders'
					},
					{
						title: 'Goods Receipts',
						url: '/procurement/goods-receipts'
					}
				]
			},
			{
				title: 'Inventory',
				url: '/inventory/stock-adjustments',
				icon: WarehouseIcon,
				items: [
					{
						title: 'Stock Adjustments',
						url: '/inventory/stock-adjustments'
					},
					{
						title: 'Stock Conversions',
						url: '/inventory/stock-conversions'
					},
					{
						title: 'Stock Report',
						url: '/inventory/stock-report'
					}
				]
			},
			{
				title: 'Payroll',
				url: '/payroll/slips',
				icon: BadgeDollarSignIcon,
				items: [
					{
						title: 'Payroll Slips',
						url: '/payroll/slips'
					},
					{
						title: 'Payroll Report',
						url: '/payroll/report'
					}
				]
			},
			{
				title: 'Reports',
				url: '/reports/daily-sales',
				icon: ChartColumnIcon,
				items: [
					{
						title: 'Daily Sales',
						url: '/reports/daily-sales'
					},
					{
						title: 'Monthly Sales',
						url: '/reports/monthly-sales'
					},
					{
						title: 'Purchase Orders',
						url: '/reports/purchase-orders'
					},
					{
						title: 'Goods Receipts',
						url: '/reports/goods-receipts'
					},
					{
						title: 'Stock Adjustments',
						url: '/reports/stock-adjustments'
					}
				]
			}
		]
	};
</script>

<script lang="ts">
	import NavMain from './nav-main.svelte';
	import NavUser from './nav-user.svelte';
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import CommandIcon from '@lucide/svelte/icons/command';
	import type { ComponentProps } from 'svelte';

	let { ref = $bindable(null), ...restProps }: ComponentProps<typeof Sidebar.Root> = $props();
</script>

<Sidebar.Root bind:ref variant="inset" {...restProps}>
	<Sidebar.Header>
		<Sidebar.Menu>
			<Sidebar.MenuItem>
				<Sidebar.MenuButton size="lg">
					{#snippet child({ props })}
						<a href="##" {...props}>
							<div
								class="flex aspect-square size-8 items-center justify-center rounded-lg bg-sidebar-primary text-sidebar-primary-foreground"
							>
								<CommandIcon class="size-4" />
							</div>
							<div class="grid flex-1 text-start text-sm leading-tight">
								<span class="truncate font-medium">POS System</span>
								<span class="truncate text-xs">Kasir & Inventory</span>
							</div>
						</a>
					{/snippet}
				</Sidebar.MenuButton>
			</Sidebar.MenuItem>
		</Sidebar.Menu>
	</Sidebar.Header>
	<Sidebar.Content>
		<NavMain items={data.navMain} />
	</Sidebar.Content>
	<Sidebar.Footer>
		<NavUser user={data.user} />
	</Sidebar.Footer>
</Sidebar.Root>
