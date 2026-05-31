<script lang="ts">
	import { onMount } from 'svelte';
	import { BarChart } from 'layerchart';
	import * as Chart from '$lib/components/ui/chart/index.js';
	import { getCashierPerformance, type CashierPerformanceItem } from '$lib/api/dashboard.js';
	import { ApiError } from '$lib/api/types.js';
	import { formatIdr } from '$lib/formatters.js';
	import { t } from '$lib/i18n/locale.svelte.js';
	import ChartCard from './chart-card.svelte';
	import RangeSelect from './range-select.svelte';
	import { formatCompact } from './chart-helpers.js';

	let days = $state(30);
	let items = $state<CashierPerformanceItem[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);

	const chartData = $derived(
		items.map((item) => ({
			cashier: item.cashierName,
			revenue: item.revenue,
			transactionCount: item.transactionCount
		}))
	);

	const chartConfig = $derived({
		revenue: { label: t('dashboard.charts.revenue'), color: 'var(--chart-5)' }
	} satisfies Chart.ChartConfig);

	const hasData = $derived(chartData.length > 0 && items.some((item) => item.revenue > 0));

	onMount(() => {
		void load();
	});

	async function load() {
		loading = true;
		error = null;
		try {
			items = await getCashierPerformance(days);
		} catch (err) {
			error = err instanceof ApiError || err instanceof Error ? err.message : 'Error';
			items = [];
		} finally {
			loading = false;
		}
	}
</script>

<ChartCard
	title={t('dashboard.charts.cashierTitle')}
	description={t('dashboard.charts.cashierDesc')}
	{loading}
	{error}
	empty={!hasData}
>
	{#snippet action()}
		<RangeSelect bind:value={days} disabled={loading} onChange={() => load()} />
	{/snippet}

	<Chart.Container config={chartConfig} class="h-[280px] w-full">
		<BarChart
			data={chartData}
			x="cashier"
			y="revenue"
			series={[{ key: 'revenue', label: t('dashboard.charts.revenue'), color: 'var(--chart-5)' }]}
			props={{
				bars: { radius: 4, rounded: 'top' },
				yAxis: { format: (value: number) => formatCompact(value) }
			}}
		>
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
