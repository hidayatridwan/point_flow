<script lang="ts" module>
	import BadgeDollarSignIcon from '@lucide/svelte/icons/badge-dollar-sign';
	import ChartColumnIcon from '@lucide/svelte/icons/chart-column';
	import DatabaseIcon from '@lucide/svelte/icons/database';
	import HeadsetIcon from '@lucide/svelte/icons/headset';
	import LayoutDashboardIcon from '@lucide/svelte/icons/layout-dashboard';
	import ScrollTextIcon from '@lucide/svelte/icons/scroll-text';
	import ShieldCheckIcon from '@lucide/svelte/icons/shield-check';
	import ShoppingCartIcon from '@lucide/svelte/icons/shopping-cart';
	import TruckIcon from '@lucide/svelte/icons/truck';
	import SettingsIcon from '@lucide/svelte/icons/settings';
	import UserCogIcon from '@lucide/svelte/icons/user-cog';
	import WalletIcon from '@lucide/svelte/icons/wallet';
	import WarehouseIcon from '@lucide/svelte/icons/warehouse';

	const data = {
		navGroups: [
			{
				titleKey: 'navGroup.overview',
				items: [
					{
						titleKey: 'nav.dashboard',
						url: '/dashboard',
						icon: LayoutDashboardIcon
					}
				]
			},
			{
				titleKey: 'navGroup.core',
				items: [
					{
						titleKey: 'nav.cashier',
						url: '/point_flow/register-session',
						icon: ShoppingCartIcon,
						items: [
							{
								titleKey: 'nav.registerSession',
								url: '/point_flow/register-session'
							},
							{
								titleKey: 'nav.newTransaction',
								url: '/point_flow/sales/new'
							},
							{
								titleKey: 'nav.salesHistory',
								url: '/point_flow/sales'
							},
							{
								titleKey: 'nav.saleReturns',
								url: '/point_flow/sale-returns'
							},
							{
								titleKey: 'nav.onlineOrders',
								url: '/point_flow/online-orders'
							},
							{
								titleKey: 'nav.cashMovements',
								url: '/point_flow/cash-drawer-movements'
							}
						]
					},
					{
						titleKey: 'nav.masterData',
						url: '/master/product-categories',
						icon: DatabaseIcon,
						items: [
							{
								titleKey: 'nav.productCategories',
								url: '/master/product-categories'
							},
							{
								titleKey: 'nav.products',
								url: '/master/products'
							},
							{
								titleKey: 'nav.units',
								url: '/master/units'
							},
							{
								titleKey: 'nav.productSkus',
								url: '/master/product-skus'
							},
							{
								titleKey: 'nav.suppliers',
								url: '/master/suppliers'
							},
							{
								titleKey: 'nav.cashRegisters',
								url: '/master/cash-registers'
							},
							{
								titleKey: 'nav.employees',
								url: '/master/employees'
							}
						]
					},
					{
						titleKey: 'nav.procurement',
						url: '/procurement/purchase-orders',
						icon: TruckIcon,
						items: [
							{
								titleKey: 'nav.purchaseOrders',
								url: '/procurement/purchase-orders'
							},
							{
								titleKey: 'nav.goodsReceipts',
								url: '/procurement/goods-receipts'
							}
						]
					},
					{
						titleKey: 'nav.inventory',
						url: '/inventory/stock-adjustments',
						icon: WarehouseIcon,
						items: [
							{
								titleKey: 'nav.stockAdjustments',
								url: '/inventory/stock-adjustments'
							},
							{
								titleKey: 'nav.stockConversions',
								url: '/inventory/stock-conversions'
							},
							{
								titleKey: 'nav.stockReport',
								url: '/inventory/stock-report'
							},
							{
								titleKey: 'nav.stockMovements',
								url: '/inventory/stock-movements'
							}
						]
					},
					{
						titleKey: 'nav.payroll',
						url: '/payroll/slips',
						icon: BadgeDollarSignIcon,
						items: [
							{
								titleKey: 'nav.payrollSlips',
								url: '/payroll/slips'
							},
							{
								titleKey: 'nav.payrollReport',
								url: '/payroll/report'
							}
						]
					},
					{
						titleKey: 'nav.finances',
						url: '/finances/transactions',
						icon: WalletIcon,
						items: [
							{
								titleKey: 'nav.financialTransactions',
								url: '/finances/transactions'
							},
							{
								titleKey: 'nav.incomeStatement',
								url: '/reports/income-statement'
							},
							{
								titleKey: 'nav.cashFlow',
								url: '/reports/cash-flow'
							},
							{
								titleKey: 'nav.financialPosition',
								url: '/reports/financial-position'
							}
						]
					},
					{
						titleKey: 'nav.reports',
						url: '/reports/daily-sales',
						icon: ChartColumnIcon,
						items: [
							{
								titleKey: 'nav.dailySales',
								url: '/reports/daily-sales'
							},
							{
								titleKey: 'nav.monthlySales',
								url: '/reports/monthly-sales'
							},
							{
								titleKey: 'nav.purchaseOrders',
								url: '/reports/purchase-orders'
							},
							{
								titleKey: 'nav.goodsReceipts',
								url: '/reports/goods-receipts'
							},
							{
								titleKey: 'nav.stockAdjustments',
								url: '/reports/stock-adjustments'
							}
						]
					}
				]
			},
			{
				titleKey: 'navGroup.config',
				items: [
					{
						titleKey: 'nav.storeSettings',
						url: '/config/store-settings',
						icon: SettingsIcon
					},
					{
						titleKey: 'nav.users',
						url: '/config/users',
						icon: UserCogIcon
					},
					{
						titleKey: 'nav.roles',
						url: '/config/roles',
						icon: ShieldCheckIcon
					},
					{
						titleKey: 'nav.auditLogs',
						url: '/config/audit-logs',
						icon: ScrollTextIcon
					},
					{
						titleKey: 'nav.helpDesk',
						url: '/help-desk',
						icon: HeadsetIcon
					}
				]
			}
		]
	};
</script>

<script lang="ts">
	import { onMount } from 'svelte';
	import NavMain from './nav-main.svelte';
	import NavUser from './nav-user.svelte';
	import { listOnlineOrders } from '$lib/api/online-orders.js';
	import { getCurrentUser } from '$lib/api/session.js';
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import CommandIcon from '@lucide/svelte/icons/command';
	import favicon from '$lib/assets/favicon.svg';
	import { t } from '$lib/i18n/locale.svelte.js';
	import type { ComponentProps } from 'svelte';

	let { ref = $bindable(null), ...restProps }: ComponentProps<typeof Sidebar.Root> = $props();

	let sidebarUser = $state<{ name: string; email: string; avatar: string } | null>(null);
	let newOrderCount = $state(0);

	const navBadges = $derived({ '/point_flow/online-orders': newOrderCount });

	onMount(() => {
		void loadCurrentUser();
		void refreshNewOrderCount();
		const badgeTimer = setInterval(() => {
			void refreshNewOrderCount();
		}, 60000);
		return () => clearInterval(badgeTimer);
	});

	async function loadCurrentUser() {
		try {
			const currentUser = await getCurrentUser();
			sidebarUser = { name: currentUser.name, email: currentUser.email, avatar: favicon };
		} catch {
			sidebarUser = null;
		}
	}

	async function refreshNewOrderCount() {
		try {
			const result = await listOnlineOrders({ status: ['new'], limit: 1, page: 1 });
			newOrderCount = result.pagination.totalRecords;
		} catch {
			newOrderCount = 0;
		}
	}
</script>

<Sidebar.Root bind:ref variant="inset" {...restProps}>
	<Sidebar.Header>
		<Sidebar.Menu>
			<Sidebar.MenuItem>
				<Sidebar.MenuButton size="lg">
					{#snippet child({ props })}
						<a href="##" {...props}>
							<div
								class="flex aspect-square size-8 items-center justify-center rounded-lg bg-sidebar-primary text-sidebar-primary-foreground dark:bg-sidebar-foreground dark:text-sidebar"
							>
								<CommandIcon class="size-4" />
							</div>
							<div class="grid flex-1 text-start text-sm leading-tight">
								<span class="truncate font-medium">{t('app.name')}</span>
								<span class="truncate text-xs">{t('app.subtitle')}</span>
							</div>
						</a>
					{/snippet}
				</Sidebar.MenuButton>
			</Sidebar.MenuItem>
		</Sidebar.Menu>
	</Sidebar.Header>
	<Sidebar.Content>
		<NavMain groups={data.navGroups} badges={navBadges} />
	</Sidebar.Content>
	<Sidebar.Footer>
		{#if sidebarUser}
			<NavUser user={sidebarUser} />
		{/if}
	</Sidebar.Footer>
</Sidebar.Root>
