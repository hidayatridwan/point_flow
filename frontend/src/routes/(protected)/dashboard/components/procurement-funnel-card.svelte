<script lang="ts">
	import { onMount } from 'svelte';
	import { getProcurementFunnel, type ProcurementFunnel } from '$lib/api/dashboard.js';
	import { ApiError } from '$lib/api/types.js';
	import { formatNumber } from '$lib/formatters.js';
	import { t } from '$lib/i18n/locale.svelte.js';
	import ChartCard from './chart-card.svelte';
	import { CHART_OTHER_COLOR, STAGE_RAMP } from './chart-helpers.js';

	type Stage = { label: string; count: number; color: string };
	type Group = { label: string; stages: Stage[] };

	let funnel = $state<ProcurementFunnel | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);

	const groups = $derived<Group[]>(funnel ? buildGroups(funnel) : []);

	const hasData = $derived(groups.some((group) => group.stages.some((stage) => stage.count > 0)));

	onMount(() => {
		void load();
	});

	async function load() {
		loading = true;
		error = null;
		try {
			funnel = await getProcurementFunnel();
		} catch (err) {
			error = err instanceof ApiError || err instanceof Error ? err.message : 'Error';
			funnel = null;
		} finally {
			loading = false;
		}
	}

	// Stages are ordinal (draft → completed), so they take one-hue ramp steps from light to
	// dark rather than unrelated categorical hues; cancelled drops out of the flow as gray.
	function buildGroups(data: ProcurementFunnel): Group[] {
		const [draft, ordered, partial, done] = STAGE_RAMP;
		const cancelled = CHART_OTHER_COLOR;
		return [
			{
				label: t('dashboard.charts.funnelPo'),
				stages: [
					{ label: t('dashboard.charts.statusDraft'), count: data.poDraft, color: draft },
					{
						label: t('dashboard.charts.statusOrdered'),
						count: data.poOrdered,
						color: ordered
					},
					{
						label: t('dashboard.charts.statusPartial'),
						count: data.poPartiallyReceived,
						color: partial
					},
					{
						label: t('dashboard.charts.statusReceived'),
						count: data.poReceived,
						color: done
					},
					{
						label: t('dashboard.charts.statusCancelled'),
						count: data.poCancelled,
						color: cancelled
					}
				]
			},
			{
				label: t('dashboard.charts.funnelGr'),
				stages: [
					{ label: t('dashboard.charts.statusDraft'), count: data.grDraft, color: draft },
					{
						label: t('dashboard.charts.statusPosted'),
						count: data.grPosted,
						color: done
					},
					{
						label: t('dashboard.charts.statusCancelled'),
						count: data.grCancelled,
						color: cancelled
					}
				]
			},
			{
				label: t('dashboard.charts.funnelSa'),
				stages: [
					{ label: t('dashboard.charts.statusDraft'), count: data.saDraft, color: draft },
					{
						label: t('dashboard.charts.statusPosted'),
						count: data.saPosted,
						color: done
					},
					{
						label: t('dashboard.charts.statusCancelled'),
						count: data.saCancelled,
						color: cancelled
					}
				]
			}
		];
	}

	function widthPercent(count: number, stages: Stage[]): number {
		const max = Math.max(...stages.map((stage) => stage.count), 1);
		return Math.max((count / max) * 100, count > 0 ? 4 : 0);
	}
</script>

<ChartCard
	title={t('dashboard.charts.funnelTitle')}
	description={t('dashboard.charts.funnelDesc')}
	{loading}
	{error}
	empty={!hasData}
>
	<div class="grid gap-6 sm:grid-cols-3">
		{#each groups as group (group.label)}
			<div class="space-y-2.5">
				<div class="text-xs font-semibold tracking-wide text-muted-foreground uppercase">
					{group.label}
				</div>
				{#each group.stages as stage (stage.label)}
					<div
						class="group/stage flex items-center gap-2"
						title={`${stage.label}: ${formatNumber(stage.count)}`}
					>
						<span class="w-24 shrink-0 truncate text-xs text-muted-foreground">{stage.label}</span>
						<div class="relative h-6 flex-1 overflow-hidden rounded-md bg-muted/60">
							<div
								class="absolute inset-y-0 left-0 rounded-md transition-all duration-300 group-hover/stage:brightness-110"
								style="width: {widthPercent(
									stage.count,
									group.stages
								)}%; background-color: {stage.color}"
							></div>
						</div>
						<span class="w-8 shrink-0 text-right font-mono text-sm font-medium tabular-nums">
							{formatNumber(stage.count)}
						</span>
					</div>
				{/each}
			</div>
		{/each}
	</div>
</ChartCard>
