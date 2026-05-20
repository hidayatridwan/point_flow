<script lang="ts">
	import '../layout.css';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { onMount } from 'svelte';
	import { getAccessToken } from '$lib/api/auth.svelte.js';
	import { refreshAccessToken } from '$lib/api/client.js';
	import favicon from '$lib/assets/favicon.svg';
	import AppSidebar from '$lib/components/app-sidebar.svelte';
	import * as Breadcrumb from '$lib/components/ui/breadcrumb/index.js';
	import { Separator } from '$lib/components/ui/separator/index.js';
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';

	type BreadcrumbItem = {
		label: string;
		href?: string;
	};

	let { children } = $props();

	const sectionLabelBySegment: Record<string, string> = {
		dashboard: 'Dashboard',
		pos: 'Point of Sale',
		master: 'Master Data',
		procurement: 'Procurement',
		inventory: 'Inventory',
		payroll: 'Payroll',
		reports: 'Reports',
		account: 'Account'
	};

	const routeLabelByPath: Record<string, string> = {
		'/dashboard': 'Ringkasan',
		'/pos/sales/new': 'Transaksi Baru',
		'/pos/sales': 'Riwayat Penjualan',
		'/master/users': 'Users',
		'/master/product-categories': 'Product Categories',
		'/master/products': 'Products',
		'/master/units': 'Units',
		'/master/product-skus': 'Product SKUs',
		'/master/suppliers': 'Suppliers',
		'/master/employees': 'Employees',
		'/procurement/purchase-orders': 'Purchase Orders',
		'/procurement/goods-receipts': 'Goods Receipts',
		'/inventory/stock-adjustments': 'Stock Adjustments',
		'/inventory/stock-conversions': 'Stock Conversions',
		'/inventory/stock-report': 'Stock Report',
		'/payroll/slips': 'Payroll Slips',
		'/payroll/report': 'Payroll Report',
		'/reports/daily-sales': 'Daily Sales',
		'/reports/monthly-sales': 'Monthly Sales',
		'/reports/purchase-orders': 'Purchase Orders',
		'/reports/goods-receipts': 'Goods Receipts',
		'/reports/stock-adjustments': 'Stock Adjustments',
		'/account/me': 'Current User'
	};

	const breadcrumbItems = $derived(buildBreadcrumbItems(page.url.pathname));

	onMount(() => {
		void ensureSession();
	});

	async function ensureSession() {
		if (getAccessToken()) {
			return;
		}

		const accessToken = await refreshAccessToken();
		if (!accessToken) {
			await goto(`/login?redirectTo=${encodeURIComponent(page.url.pathname + page.url.search)}`, {
				replaceState: true
			});
			return;
		}
	}

	function buildBreadcrumbItems(pathname: string): BreadcrumbItem[] {
		const normalizedPath = normalizePath(pathname);
		const segments = normalizedPath.split('/').filter(Boolean);

		if (segments.length === 0) {
			return [{ label: 'Dashboard', href: '/dashboard' }];
		}

		const sectionLabel = sectionLabelBySegment[segments[0]] ?? formatSegmentLabel(segments[0]);
		const routeLabel = routeLabelByPath[normalizedPath];

		if (routeLabel) {
			return routeLabel === sectionLabel
				? [{ label: sectionLabel }]
				: [{ label: sectionLabel }, { label: routeLabel }];
		}

		return segments.map((segment, index) => {
			const href = `/${segments.slice(0, index + 1).join('/')}`;
			const label = routeLabelByPath[href] ?? sectionLabelBySegment[segment] ?? formatSegmentLabel(segment);
			const isLast = index === segments.length - 1;

			return {
				label,
				href: !isLast && routeLabelByPath[href] ? href : undefined
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
</script>

<svelte:head><link rel="icon" href={favicon} /></svelte:head>

<Sidebar.Provider>
	<AppSidebar />
	<Sidebar.Inset>
		<header class="flex h-16 shrink-0 items-center gap-2">
			<div class="flex items-center gap-2 px-4">
				<Sidebar.Trigger class="-ms-1" />
				<Separator orientation="vertical" class="me-2 data-[orientation=vertical]:h-4" />
				<Breadcrumb.Root>
					<Breadcrumb.List>
						{#each breadcrumbItems as breadcrumb, index (`${index}-${breadcrumb.label}`)}
							<Breadcrumb.Item class={index < breadcrumbItems.length - 1 ? 'hidden md:block' : undefined}>
								{#if index === breadcrumbItems.length - 1}
									<Breadcrumb.Page>{breadcrumb.label}</Breadcrumb.Page>
								{:else if breadcrumb.href}
									<Breadcrumb.Link href={breadcrumb.href}>{breadcrumb.label}</Breadcrumb.Link>
								{:else}
									<span>{breadcrumb.label}</span>
								{/if}
							</Breadcrumb.Item>
							{#if index < breadcrumbItems.length - 1}
								<Breadcrumb.Separator class="hidden md:block" />
							{/if}
						{/each}
					</Breadcrumb.List>
				</Breadcrumb.Root>
			</div>
		</header>
		<div class="flex flex-1 flex-col gap-4 p-4 pt-0">
			{@render children()}
		</div>
	</Sidebar.Inset>
</Sidebar.Provider>
