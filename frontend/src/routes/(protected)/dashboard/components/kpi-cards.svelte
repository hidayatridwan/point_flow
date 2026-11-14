<script lang="ts">
	import type { Component } from 'svelte';
	import BanknoteIcon from '@lucide/svelte/icons/banknote';
	import ClipboardListIcon from '@lucide/svelte/icons/clipboard-list';
	import PackageSearchIcon from '@lucide/svelte/icons/package-search';
	import ReceiptTextIcon from '@lucide/svelte/icons/receipt-text';
	import TruckIcon from '@lucide/svelte/icons/truck';
	import UsersIcon from '@lucide/svelte/icons/users';
	import * as Card from '$lib/components/ui/card/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import { formatIdr, formatNumber } from '$lib/formatters.js';
	import { t } from '$lib/i18n/locale.svelte.js';
	import type { DashboardSummary } from '$lib/api/dashboard.js';

	let { summary, loading = false }: { summary: DashboardSummary | null; loading?: boolean } =
		$props();

	// `alert` is reserved for metrics that signal a problem right now; everything else
	// wears the brand tint so color keeps its meaning.
	type Tone = 'brand' | 'alert';

	type Metric = {
		label: string;
		value: string;
		description: string;
		icon: Component;
		tone: Tone;
	};

	const toneClasses: Record<Tone, string> = {
		brand: 'bg-chart-1/10 text-chart-1 dark:bg-chart-1/20',
		alert: 'bg-destructive/10 text-destructive dark:bg-destructive/20'
	};

	const metrics = $derived<Metric[]>(
		summary
			? [
					{
						label: t('dashboard.revenueToday'),
						value: formatIdr(summary.revenueToday),
						description: `${t('dashboard.revenueDesc')} ${t('dashboard.cashRevenue')}: ${formatIdr(summary.cashRevenueToday)} • ${t('dashboard.noncashRevenue')}: ${formatIdr(summary.noncashRevenueToday)}`,
						icon: BanknoteIcon,
						tone: 'brand'
					},
					{
						label: t('dashboard.transactions'),
						value: formatNumber(summary.transactionCountToday),
						description: t('dashboard.transactionsDesc'),
						icon: ReceiptTextIcon,
						tone: 'brand'
					},
					{
						label: t('dashboard.lowStock'),
						value: formatNumber(summary.lowStockProductCount),
						description: t('dashboard.lowStockDesc'),
						icon: PackageSearchIcon,
						tone: summary.lowStockProductCount > 0 ? 'alert' : 'brand'
					},
					{
						label: t('dashboard.pendingPO'),
						value: formatNumber(summary.pendingPurchaseOrderCount),
						description: t('dashboard.pendingPODesc'),
						icon: TruckIcon,
						tone: 'brand'
					},
					{
						label: t('dashboard.draftAdj'),
						value: formatNumber(summary.draftStockAdjustmentCount),
						description: t('dashboard.draftAdjDesc'),
						icon: ClipboardListIcon,
						tone: 'brand'
					},
					{
						label: t('dashboard.activeEmployees'),
						value: formatNumber(summary.activeEmployeeCount),
						description: t('dashboard.activeEmployeesDesc'),
						icon: UsersIcon,
						tone: 'brand'
					}
				]
			: []
	);
</script>

<section class="grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
	{#if loading}
		{#each Array.from({ length: 6 }).keys() as index (index)}
			<Skeleton class="h-[116px] w-full rounded-xl" />
		{/each}
	{:else}
		{#each metrics as metric (metric.label)}
			{@const Icon = metric.icon}
			<Card.Root size="sm" class="border border-border/70 shadow-none ring-0">
				<Card.Header class="flex flex-row items-center justify-between gap-3">
					<div class="space-y-1">
						<Card.Description>{metric.label}</Card.Description>
						<Card.Title class="text-2xl">{metric.value}</Card.Title>
					</div>
					<div
						class="flex size-10 shrink-0 items-center justify-center rounded-full {toneClasses[
							metric.tone
						]}"
					>
						<Icon class="size-5" />
					</div>
				</Card.Header>
				<Card.Content>
					<p class="text-sm text-muted-foreground">{metric.description}</p>
				</Card.Content>
			</Card.Root>
		{/each}
	{/if}
</section>
