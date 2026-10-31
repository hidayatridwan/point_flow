<script lang="ts">
	import { onMount } from 'svelte';
	import CircleAlertIcon from '@lucide/svelte/icons/circle-alert';
	import { getDashboardSummary, type DashboardSummary } from '$lib/api/dashboard.js';
	import { ApiError } from '$lib/api/types.js';
	import * as Alert from '$lib/components/ui/alert/index.js';
	import { t } from '$lib/i18n/locale.svelte.js';
	import KpiCards from './components/kpi-cards.svelte';
	import SalesTrendCard from './components/sales-trend-card.svelte';
	import HourlySalesCard from './components/hourly-sales-card.svelte';
	import TopProductsCard from './components/top-products-card.svelte';
	import CategoryChartCard from './components/category-chart-card.svelte';
	import ProductScatterCard from './components/product-scatter-card.svelte';
	import CashierChartCard from './components/cashier-chart-card.svelte';
	import ProcurementFunnelCard from './components/procurement-funnel-card.svelte';
	import LowStockCard from './components/low-stock-card.svelte';

	let summary = $state<DashboardSummary | null>(null);
	let summaryLoading = $state(true);
	let summaryError = $state<string | null>(null);

	onMount(() => {
		void loadSummary();
	});

	async function loadSummary() {
		summaryLoading = true;
		summaryError = null;
		try {
			summary = await getDashboardSummary();
		} catch (error) {
			summaryError =
				error instanceof ApiError || error instanceof Error
					? error.message
					: t('dashboard.errorLoad');
			summary = null;
		} finally {
			summaryLoading = false;
		}
	}
</script>

<svelte:head>
	<title>{t('dashboard.title')} | PointFlow</title>
</svelte:head>

<div class="flex flex-col gap-6">
	<div class="flex flex-col gap-1">
		<h1 class="text-xl font-semibold tracking-tight">{t('dashboard.title')}</h1>
		<p class="max-w-2xl text-sm text-muted-foreground">{t('dashboard.subtitle')}</p>
	</div>

	{#if summaryError}
		<Alert.Root variant="destructive">
			<CircleAlertIcon />
			<Alert.Title>{t('dashboard.errorTitle')}</Alert.Title>
			<Alert.Description>{summaryError}</Alert.Description>
		</Alert.Root>
	{:else}
		<KpiCards {summary} loading={summaryLoading} />
	{/if}

	<!-- Time-series: revenue trend + today's hourly distribution -->
	<section class="grid gap-4 lg:grid-cols-2">
		<SalesTrendCard />
		<HourlySalesCard />
	</section>

	<!-- Composition + ranking: category mix and best sellers -->
	<section class="grid gap-4 lg:grid-cols-2">
		<CategoryChartCard />
		<TopProductsCard />
	</section>

	<!-- Relationships + people: price/volume scatter and cashier output -->
	<section class="grid gap-4 lg:grid-cols-2">
		<ProductScatterCard />
		<CashierChartCard />
	</section>

	<!-- Pipeline + watchlist: procurement funnel and low-stock table -->
	<section class="grid gap-4 lg:grid-cols-2">
		<ProcurementFunnelCard />
		<LowStockCard />
	</section>
</div>
