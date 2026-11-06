<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import { onMount } from 'svelte';
	import { getAccessToken } from '$lib/api/auth.svelte.js';
	import { refreshAccessToken } from '$lib/api/client.js';
	import AppSidebar from '$lib/components/app-sidebar.svelte';
	import LanguageToggle from '$lib/components/language-toggle.svelte';
	import SessionTransition from '$lib/components/session-transition.svelte';
	import ThemeToggle from '$lib/components/theme-toggle.svelte';
	import * as Breadcrumb from '$lib/components/ui/breadcrumb/index.js';
	import { Separator } from '$lib/components/ui/separator/index.js';
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import { t } from '$lib/i18n/locale.svelte.js';

	type BreadcrumbItem = {
		label?: string;
		labelKey?: string;
		href?: string;
	};

	let { children } = $props();

	const sectionLabelKeyBySegment: Record<string, string> = {
		dashboard: 'nav.dashboard',
		point_flow: 'nav.pointFlow',
		master: 'nav.masterData',
		procurement: 'nav.procurement',
		inventory: 'nav.inventory',
		payroll: 'nav.payroll',
		reports: 'nav.reports',
		config: 'nav.config',
		account: 'nav.account',
		'help-desk': 'nav.helpDesk'
	};

	const routeLabelKeyByPath: Record<string, string> = {
		'/dashboard': 'nav.summary',
		'/point_flow/register-session': 'nav.registerSession',
		'/point_flow/sales/new': 'nav.newTransaction',
		'/point_flow/sales': 'nav.salesHistory',
		'/point_flow/sale-returns': 'nav.saleReturns',
		'/point_flow/cash-drawer-movements': 'nav.cashMovements',
		'/config/users': 'nav.users',
		'/master/product-categories': 'nav.productCategories',
		'/master/products': 'nav.products',
		'/master/units': 'nav.units',
		'/master/product-skus': 'nav.productSkus',
		'/master/suppliers': 'nav.suppliers',
		'/master/cash-registers': 'nav.cashRegisters',
		'/master/employees': 'nav.employees',
		'/procurement/purchase-orders': 'nav.purchaseOrders',
		'/procurement/goods-receipts': 'nav.goodsReceipts',
		'/inventory/stock-adjustments': 'nav.stockAdjustments',
		'/inventory/stock-conversions': 'nav.stockConversions',
		'/inventory/stock-report': 'nav.stockReport',
		'/inventory/stock-movements': 'nav.stockMovements',
		'/payroll/slips': 'nav.payrollSlips',
		'/payroll/report': 'nav.payrollReport',
		'/reports/daily-sales': 'nav.dailySales',
		'/reports/monthly-sales': 'nav.monthlySales',
		'/reports/purchase-orders': 'nav.purchaseOrders',
		'/reports/goods-receipts': 'nav.goodsReceipts',
		'/reports/stock-adjustments': 'nav.stockAdjustments',
		'/config/roles': 'nav.roles',
		'/config/audit-logs': 'nav.auditLogs',
		'/account/me': 'nav.currentUser',
		'/help-desk': 'nav.helpDesk'
	};

	const breadcrumbItems = $derived(buildBreadcrumbItems(page.url.pathname));
	let isSessionReady = $state(false);
	let isRedirecting = $state(false);

	onMount(() => {
		void ensureSession();
	});

	async function ensureSession() {
		if (getAccessToken()) {
			isSessionReady = true;
			return;
		}

		const accessToken = await refreshAccessToken();
		if (!accessToken) {
			isRedirecting = true;
			const loginPath =
				`/?redirectTo=${encodeURIComponent(page.url.pathname + page.url.search)}` as const;
			await goto(resolve(loginPath), {
				replaceState: true
			});
			return;
		}

		isSessionReady = true;
	}

	function buildBreadcrumbItems(pathname: string): BreadcrumbItem[] {
		const normalizedPath = normalizePath(pathname);
		const segments = normalizedPath.split('/').filter(Boolean);

		if (segments.length === 0) {
			return [{ labelKey: 'nav.dashboard', href: '/dashboard' }];
		}

		const sectionLabelKey = sectionLabelKeyBySegment[segments[0]];
		const routeLabelKey = routeLabelKeyByPath[normalizedPath];

		if (routeLabelKey) {
			const sectionItem = sectionLabelKey
				? { labelKey: sectionLabelKey }
				: { label: formatSegmentLabel(segments[0]) };

			return routeLabelKey === sectionLabelKey
				? [sectionItem]
				: [sectionItem, { labelKey: routeLabelKey }];
		}

		return segments.map((segment, index) => {
			const href = `/${segments.slice(0, index + 1).join('/')}`;
			const labelKey = routeLabelKeyByPath[href] ?? sectionLabelKeyBySegment[segment];
			const isLast = index === segments.length - 1;

			return {
				...(labelKey ? { labelKey } : { label: formatSegmentLabel(segment) }),
				href: !isLast && routeLabelKeyByPath[href] ? href : undefined
			};
		});
	}

	function normalizePath(pathname: string) {
		if (pathname.length <= 1) return pathname;
		return pathname.replace(/\/+$/, '');
	}

	function formatSegmentLabel(segment: string) {
		return segment
			.split('-')
			.filter(Boolean)
			.map((word) => word.charAt(0).toUpperCase() + word.slice(1))
			.join(' ');
	}

	function getBreadcrumbLabel(breadcrumb: BreadcrumbItem) {
		return breadcrumb.labelKey ? t(breadcrumb.labelKey) : (breadcrumb.label ?? '');
	}
</script>

{#if isSessionReady}
	<Sidebar.Provider>
		<AppSidebar />
		<Sidebar.Inset>
			<header class="flex h-16 shrink-0 items-center justify-between gap-2 px-4">
				<div class="flex items-center gap-2">
					<Sidebar.Trigger class="-ms-1" />
					<Separator orientation="vertical" class="me-2 data-[orientation=vertical]:h-4" />
					<Breadcrumb.Root>
						<Breadcrumb.List>
							{#each breadcrumbItems as breadcrumb, index (`${index}-${breadcrumb.labelKey ?? breadcrumb.label}`)}
								<Breadcrumb.Item
									class={index < breadcrumbItems.length - 1 ? 'hidden md:block' : undefined}
								>
									{#if index === breadcrumbItems.length - 1}
										<Breadcrumb.Page>{getBreadcrumbLabel(breadcrumb)}</Breadcrumb.Page>
									{:else if breadcrumb.href}
										<Breadcrumb.Link href={breadcrumb.href}>
											{getBreadcrumbLabel(breadcrumb)}
										</Breadcrumb.Link>
									{:else}
										<span>{getBreadcrumbLabel(breadcrumb)}</span>
									{/if}
								</Breadcrumb.Item>
								{#if index < breadcrumbItems.length - 1}
									<Breadcrumb.Separator class="hidden md:block" />
								{/if}
							{/each}
						</Breadcrumb.List>
					</Breadcrumb.Root>
				</div>
				<div class="flex items-center gap-1">
					<LanguageToggle />
					<ThemeToggle />
				</div>
			</header>
			<div class="flex flex-1 flex-col gap-4 p-4 pt-0">
				{@render children()}
			</div>
		</Sidebar.Inset>
	</Sidebar.Provider>
{:else}
	<SessionTransition
		class="min-h-svh bg-muted p-6 md:p-10"
		title={isRedirecting ? t('session.openingSignIn') : t('session.checkingSession')}
		description={t('session.pleaseWait')}
	/>
{/if}
