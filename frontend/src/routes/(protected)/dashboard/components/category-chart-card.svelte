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
	import { CHART_COLORS, CHART_OTHER_COLOR, formatPercent } from './chart-helpers.js';

	type Slice = { category: string; revenue: number; percentage: number; color: string };

	let days = $state(30);
	let items = $state<SalesByCategoryItem[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);

	const chartData = $derived<Slice[]>(buildSlices(items));

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

	// Palette colors are assigned in order and never cycled: when there are more categories
	// than colors, the top earners keep their colors and the tail folds into one gray
	// "Other" slice, so no two slices ever share a color.
	function buildSlices(rows: SalesByCategoryItem[]): Slice[] {
		const sorted = [...rows].sort((a, b) => b.revenue - a.revenue);
		const namedCount =
			sorted.length > CHART_COLORS.length ? CHART_COLORS.length - 1 : sorted.length;

		const slices: Slice[] = sorted.slice(0, namedCount).map((row, index) => ({
			category: row.categoryName,
			revenue: row.revenue,
			percentage: row.percentage,
			color: CHART_COLORS[index]
		}));

		const rest = sorted.slice(namedCount);
		if (rest.length > 0) {
			slices.push({
				category: t('dashboard.charts.otherCategory'),
				revenue: rest.reduce((sum, row) => sum + row.revenue, 0),
				percentage: rest.reduce((sum, row) => sum + row.percentage, 0),
				color: CHART_OTHER_COLOR
			});
		}

		return slices;
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

	<div class="flex flex-col gap-5">
		<Chart.Container config={chartConfig} class="mx-auto aspect-square h-[240px]">
			<PieChart
				data={chartData}
				key="category"
				value="revenue"
				label="category"
				cRange={chartData.map((row) => row.color)}
				innerRadius={0.65}
				cornerRadius={4}
				padAngle={0.02}
			>
				{#snippet tooltip()}
					<Chart.Tooltip hideLabel nameKey="category">
						{#snippet formatter({ value, item })}
							<div class="flex w-full items-center justify-between gap-4">
								<span class="flex items-center gap-1.5 text-muted-foreground">
									<span class="size-2.5 rounded-[2px]" style="background-color: {item.color}"
									></span>
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

		<!-- Legend with visible shares: identity never rests on color alone, and the
		     lighter slices (amber, sky) stay readable without hovering. -->
		<ul class="grid gap-x-6 gap-y-2 text-sm sm:grid-cols-2">
			{#each chartData as slice (slice.category)}
				<li class="flex items-center justify-between gap-3">
					<span class="flex min-w-0 items-center gap-2 text-muted-foreground">
						<span class="size-2.5 shrink-0 rounded-[2px]" style="background-color: {slice.color}"
						></span>
						<span class="truncate" title={slice.category}>{slice.category}</span>
					</span>
					<span class="shrink-0 font-medium text-foreground tabular-nums">
						{formatPercent(slice.percentage)}
					</span>
				</li>
			{/each}
		</ul>
	</div>
</ChartCard>
