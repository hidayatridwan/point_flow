<script lang="ts">
	import { onMount } from 'svelte';
	import { ScatterChart } from 'layerchart';
	import * as Chart from '$lib/components/ui/chart/index.js';
	import { getProductScatter, type ProductScatterItem } from '$lib/api/dashboard.js';
	import { ApiError } from '$lib/api/types.js';
	import { formatIdr, formatNumber } from '$lib/formatters.js';
	import { t } from '$lib/i18n/locale.svelte.js';
	import ChartCard from './chart-card.svelte';
	import RangeSelect from './range-select.svelte';
	import { formatCompact } from './chart-helpers.js';

	type ScatterDatum = {
		productName: string;
		unit: string;
		sellingPrice: number;
		quantitySold: number;
		revenue: number;
	};

	let days = $state(30);
	let items = $state<ProductScatterItem[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);

	const chartData = $derived<ScatterDatum[]>(
		items.map((item) => ({
			productName: item.productName,
			unit: item.unit,
			sellingPrice: item.sellingPrice,
			quantitySold: item.quantitySold,
			revenue: item.revenue
		}))
	);

	const chartConfig = $derived({
		quantitySold: { label: t('dashboard.charts.scatterQuantity'), color: 'var(--chart-1)' }
	} satisfies Chart.ChartConfig);

	const hasData = $derived(chartData.length > 0);

	onMount(() => {
		void load();
	});

	async function load() {
		loading = true;
		error = null;
		try {
			items = await getProductScatter(days, 60);
		} catch (err) {
			error = err instanceof ApiError || err instanceof Error ? err.message : 'Error';
			items = [];
		} finally {
			loading = false;
		}
	}
</script>

<ChartCard
	title={t('dashboard.charts.scatterTitle')}
	description={t('dashboard.charts.scatterDesc')}
	{loading}
	{error}
	empty={!hasData}
>
	{#snippet action()}
		<RangeSelect bind:value={days} disabled={loading} onChange={() => load()} />
	{/snippet}

	<Chart.Container config={chartConfig} class="h-[280px] w-full">
		<ScatterChart
			data={chartData}
			x="sellingPrice"
			y="quantitySold"
			r="revenue"
			rRange={[5, 24]}
			props={{
				// Single series → slot-1 indigo; a surface-colored ring keeps overlapping bubbles apart.
				points: {
					fill: 'var(--chart-1)',
					fillOpacity: 0.5,
					stroke: 'var(--card)',
					strokeWidth: 1.5
				},
				xAxis: { format: (value: number) => formatCompact(value) },
				yAxis: { format: (value: number) => formatNumber(value) }
			}}
		>
			{#snippet tooltip({ context })}
				{@const point = context?.tooltip?.data as ScatterDatum | null | undefined}
				{#if point}
					<div
						class="grid min-w-[11rem] gap-1.5 rounded-lg border border-border/50 bg-background px-2.5 py-1.5 text-xs shadow-xl"
					>
						<div class="font-medium">{point.productName}</div>
						<div class="flex items-center justify-between gap-4">
							<span class="text-muted-foreground">{t('dashboard.charts.scatterPrice')}</span>
							<span class="font-mono font-medium tabular-nums">{formatIdr(point.sellingPrice)}</span
							>
						</div>
						<div class="flex items-center justify-between gap-4">
							<span class="text-muted-foreground">{t('dashboard.charts.scatterQuantity')}</span>
							<span class="font-mono font-medium tabular-nums">
								{formatNumber(point.quantitySold)}
								{point.unit}
							</span>
						</div>
						<div class="flex items-center justify-between gap-4">
							<span class="text-muted-foreground">{t('dashboard.charts.revenue')}</span>
							<span class="font-mono font-medium tabular-nums">{formatIdr(point.revenue)}</span>
						</div>
					</div>
				{/if}
			{/snippet}
		</ScatterChart>
	</Chart.Container>
</ChartCard>
