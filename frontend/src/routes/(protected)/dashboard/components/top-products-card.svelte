<script lang="ts">
	import { onMount } from 'svelte';
	import { getTopProducts, type TopProductItem } from '$lib/api/dashboard.js';
	import { ApiError } from '$lib/api/types.js';
	import { formatIdr, formatNumber } from '$lib/formatters.js';
	import { t } from '$lib/i18n/locale.svelte.js';
	import ChartCard from './chart-card.svelte';
	import RangeSelect from './range-select.svelte';

	let days = $state(30);
	let items = $state<TopProductItem[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);

	const hasData = $derived(items.length > 0 && items.some((item) => item.revenue > 0));

	// Largest revenue in the set, used to scale each bar's fill width.
	const maxRevenue = $derived(items.reduce((max, item) => Math.max(max, item.revenue), 0));

	// Bar fill as a percentage of the top earner; keep non-zero rows visible with a small floor.
	function fillPercent(revenue: number): number {
		if (maxRevenue <= 0 || revenue <= 0) return 0;
		return Math.max((revenue / maxRevenue) * 100, 2);
	}

	onMount(() => {
		void load();
	});

	async function load() {
		loading = true;
		error = null;
		try {
			items = await getTopProducts(days, 10);
		} catch (err) {
			error = err instanceof ApiError || err instanceof Error ? err.message : 'Error';
			items = [];
		} finally {
			loading = false;
		}
	}
</script>

<ChartCard
	title={t('dashboard.charts.topProductsTitle')}
	description={t('dashboard.charts.topProductsDesc')}
	{loading}
	{error}
	empty={!hasData}
>
	{#snippet action()}
		<RangeSelect bind:value={days} disabled={loading} onChange={() => load()} />
	{/snippet}

	<ol class="flex flex-col gap-3">
		{#each items as item, index (item.productSkuId)}
			<li class="group flex flex-col gap-1.5">
				<div class="flex items-baseline justify-between gap-3">
					<div class="flex min-w-0 items-baseline gap-2.5">
						<span
							class="w-4 shrink-0 text-right text-xs font-semibold text-muted-foreground tabular-nums"
						>
							{index + 1}
						</span>
						<span class="truncate text-sm font-medium" title={item.productName}>
							{item.productName}
						</span>
					</div>
					<span class="shrink-0 font-mono text-sm font-semibold tabular-nums">
						{formatIdr(item.revenue)}
					</span>
				</div>
				<div class="flex items-center gap-3 pl-6.5">
					<div class="h-2 flex-1 overflow-hidden rounded-full bg-muted">
						<div
							class="h-full rounded-full transition-[width] duration-500 ease-out"
							style="width: {fillPercent(item.revenue)}%; background-color: var(--chart-2)"
						></div>
					</div>
					<span class="shrink-0 text-xs text-muted-foreground tabular-nums">
						{formatNumber(item.quantitySold)}
						{t('dashboard.charts.topProductsSold')}
					</span>
				</div>
			</li>
		{/each}
	</ol>
</ChartCard>
