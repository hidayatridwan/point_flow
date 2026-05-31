<script lang="ts">
	import { onMount } from 'svelte';
	import { AreaChart } from 'layerchart';
	import * as Chart from '$lib/components/ui/chart/index.js';
	import { getSalesTrend, type SalesTrendPoint } from '$lib/api/dashboard.js';
	import { ApiError } from '$lib/api/types.js';
	import { formatIdr } from '$lib/formatters.js';
	import { t } from '$lib/i18n/locale.svelte.js';
	import ChartCard from './chart-card.svelte';
	import RangeSelect from './range-select.svelte';
	import { formatCompact, formatShortDate } from './chart-helpers.js';

	let days = $state(30);
	let points = $state<SalesTrendPoint[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);

	const chartData = $derived(
		points.map((point) => ({
			date: new Date(`${point.date}T00:00:00`),
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
			points = await getSalesTrend(days);
		} catch (err) {
			error = err instanceof ApiError || err instanceof Error ? err.message : 'Error';
			points = [];
		} finally {
			loading = false;
		}
	}
</script>

<ChartCard
	title={t('dashboard.charts.salesTrendTitle')}
	description={t('dashboard.charts.salesTrendDesc')}
	{loading}
	{error}
	empty={!hasData}
>
	{#snippet action()}
		<RangeSelect bind:value={days} disabled={loading} onChange={() => load()} />
	{/snippet}

	<Chart.Container config={chartConfig} class="h-[260px] w-full">
		<AreaChart
			data={chartData}
			x="date"
			y="revenue"
			series={[{ key: 'revenue', label: t('dashboard.charts.revenue'), color: 'var(--chart-1)' }]}
			props={{
				area: { fillOpacity: 0.25, line: { width: 2 } },
				xAxis: { format: (value: Date) => formatShortDate(value) },
				yAxis: { format: (value: number) => formatCompact(value) }
			}}
		>
			{#snippet tooltip()}
				<Chart.Tooltip indicator="line" labelFormatter={(value: Date) => formatShortDate(value)}>
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
		</AreaChart>
	</Chart.Container>
</ChartCard>
