<script lang="ts">
	import { onMount } from 'svelte';
	import { BarChart, Bars, LinearGradient } from 'layerchart';
	import * as Chart from '$lib/components/ui/chart/index.js';
	import { getHourlySales, type HourlySalesPoint } from '$lib/api/dashboard.js';
	import { ApiError } from '$lib/api/types.js';
	import { formatIdr } from '$lib/formatters.js';
	import { t } from '$lib/i18n/locale.svelte.js';
	import ChartCard from './chart-card.svelte';
	import { BAR_GRADIENT_STOPS, formatCompact, formatHour } from './chart-helpers.js';

	let points = $state<HourlySalesPoint[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);

	const chartData = $derived(
		points.map((point) => ({
			hour: formatHour(point.hour),
			revenue: point.revenue,
			transactionCount: point.transactionCount
		}))
	);

	const chartConfig = $derived({
		revenue: { label: t('dashboard.charts.revenue'), color: 'var(--chart-1)' }
	} satisfies Chart.ChartConfig);

	const hasData = $derived(points.some((point) => point.revenue > 0));

	onMount(() => {
		void load();
	});

	async function load() {
		loading = true;
		error = null;
		try {
			points = await getHourlySales();
		} catch (err) {
			error = err instanceof ApiError || err instanceof Error ? err.message : 'Error';
			points = [];
		} finally {
			loading = false;
		}
	}
</script>

<ChartCard
	title={t('dashboard.charts.hourlySalesTitle')}
	description={t('dashboard.charts.hourlySalesDesc')}
	{loading}
	{error}
	empty={!hasData}
>
	<Chart.Container config={chartConfig} class="h-[260px] w-full">
		<BarChart
			data={chartData}
			x="hour"
			y="revenue"
			series={[{ key: 'revenue', label: t('dashboard.charts.revenue'), color: 'var(--chart-1)' }]}
			props={{
				xAxis: { ticks: 8 },
				yAxis: { format: (value: number) => formatCompact(value) }
			}}
		>
			<!-- Each bar gets its own indigo → purple gradient (objectBoundingBox units). -->
			{#snippet marks()}
				<LinearGradient stops={BAR_GRADIENT_STOPS} vertical>
					{#snippet children({ gradient })}
						<Bars seriesKey="revenue" fill={gradient} radius={6} rounded="edge" strokeWidth={0} />
					{/snippet}
				</LinearGradient>
			{/snippet}
			{#snippet tooltip()}
				<Chart.Tooltip indicator="dot">
					{#snippet formatter({ value, item })}
						<div class="flex w-full items-center justify-between gap-4">
							<span class="flex items-center gap-1.5 text-muted-foreground">
								<span class="size-2.5 rounded-[2px]" style="background-color: {item.color}"></span>
								{item.label}
							</span>
							<span class="font-mono font-medium text-foreground tabular-nums">
								{formatIdr(Number(value))}
							</span>
						</div>
					{/snippet}
				</Chart.Tooltip>
			{/snippet}
		</BarChart>
	</Chart.Container>
</ChartCard>
