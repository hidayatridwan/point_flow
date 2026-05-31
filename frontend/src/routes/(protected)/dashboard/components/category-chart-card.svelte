<script lang="ts">
	import { onMount } from 'svelte';
	import { PieChart } from 'layerchart';
	import * as Chart from '$lib/components/ui/chart/index.js';
	import { getSalesByCategory, type SalesByCategoryItem } from '$lib/api/dashboard.js';
	import { ApiError } from '$lib/api/types.js';
	import { formatIdr } from '$lib/formatters.js';
	import { t } from '$lib/i18n/locale.svelte.js';
	import ChartCard from './chart-card.svelte';
	import RangeSelect from './range-select.svelte';
	import { CHART_COLORS } from './chart-helpers.js';

	let days = $state(30);
	let items = $state<SalesByCategoryItem[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);

	const chartData = $derived(
		items.map((item, index) => ({
			category: item.categoryName,
			revenue: item.revenue,
			percentage: item.percentage,
			color: CHART_COLORS[index % CHART_COLORS.length]
		}))
	);

	// Config keyed by category so the tooltip/legend resolve labels and colors per slice.
	const chartConfig = $derived(
		Object.fromEntries(
			chartData.map((row) => [row.category, { label: row.category, color: row.color }])
		) satisfies Chart.ChartConfig
	);

	const hasData = $derived(items.length > 0 && items.some((item) => item.revenue > 0));

	onMount(() => {
		void load();
	});

	async function load() {
		loading = true;
		error = null;
		try {
			items = await getSalesByCategory(days);
		} catch (err) {
			error = err instanceof ApiError || err instanceof Error ? err.message : 'Error';
			items = [];
		} finally {
			loading = false;
		}
	}
</script>

<ChartCard
	title={t('dashboard.charts.categoryTitle')}
	description={t('dashboard.charts.categoryDesc')}
	{loading}
	{error}
	empty={!hasData}
>
	{#snippet action()}
		<RangeSelect bind:value={days} disabled={loading} onChange={() => load()} />
	{/snippet}

	<Chart.Container config={chartConfig} class="mx-auto aspect-square h-[280px]">
		<PieChart
			data={chartData}
			key="category"
			value="revenue"
			label="category"
			cRange={chartData.map((row) => row.color)}
			innerRadius={0.65}
			cornerRadius={3}
			padAngle={0.02}
			legend
			props={{ legend: { placement: 'bottom' } }}
		>
			{#snippet tooltip()}
				<Chart.Tooltip hideLabel nameKey="category">
					{#snippet formatter({ value, item })}
						<div class="flex w-full items-center justify-between gap-4">
							<span class="flex items-center gap-1.5 text-muted-foreground">
								<span class="size-2.5 rounded-[2px]" style="background-color: {item.color}"></span>
								{item.label ?? item.key}
							</span>
							<span class="font-mono font-medium text-foreground tabular-nums">
								{formatIdr(Number(value))}
							</span>
						</div>
					{/snippet}
				</Chart.Tooltip>
			{/snippet}
		</PieChart>
	</Chart.Container>
</ChartCard>
