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

	type Metric = {
		label: string;
		value: string;
		description: string;
		icon: Component;
		tone: string;
		accent: string;
	};

	const metrics = $derived<Metric[]>(
		summary
			? [
					{
						label: t('dashboard.revenueToday'),
						value: formatIdr(summary.revenueToday),
						description: t('dashboard.revenueDesc'),
						icon: BanknoteIcon,
						tone: 'bg-emerald-500/10 text-emerald-700 dark:bg-emerald-400/10 dark:text-emerald-300',
						accent: 'var(--color-emerald-500, #10b981)'
					},
					{
						label: t('dashboard.transactions'),
						value: formatNumber(summary.transactionCountToday),
						description: t('dashboard.transactionsDesc'),
						icon: ReceiptTextIcon,
						tone: 'bg-sky-500/10 text-sky-700 dark:bg-sky-400/10 dark:text-sky-300',
						accent: 'var(--color-sky-500, #0ea5e9)'
					},
					{
						label: t('dashboard.lowStock'),
						value: formatNumber(summary.lowStockProductCount),
						description: t('dashboard.lowStockDesc'),
						icon: PackageSearchIcon,
						tone: 'bg-amber-500/10 text-amber-700 dark:bg-amber-400/10 dark:text-amber-300',
						accent: 'var(--color-amber-500, #f59e0b)'
					},
					{
						label: t('dashboard.pendingPO'),
						value: formatNumber(summary.pendingPurchaseOrderCount),
						description: t('dashboard.pendingPODesc'),
						icon: TruckIcon,
						tone: 'bg-violet-500/10 text-violet-700 dark:bg-violet-400/10 dark:text-violet-300',
						accent: 'var(--color-violet-500, #8b5cf6)'
					},
					{
						label: t('dashboard.draftAdj'),
						value: formatNumber(summary.draftStockAdjustmentCount),
						description: t('dashboard.draftAdjDesc'),
						icon: ClipboardListIcon,
						tone: 'bg-rose-500/10 text-rose-700 dark:bg-rose-400/10 dark:text-rose-300',
						accent: 'var(--color-rose-500, #f43f5e)'
					},
					{
						label: t('dashboard.activeEmployees'),
						value: formatNumber(summary.activeEmployeeCount),
						description: t('dashboard.activeEmployeesDesc'),
						icon: UsersIcon,
						tone: 'bg-cyan-500/10 text-cyan-700 dark:bg-cyan-400/10 dark:text-cyan-300',
						accent: 'var(--color-cyan-500, #06b6d4)'
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
			<Card.Root
				size="sm"
				style={`--tile: ${metric.accent}`}
				class="dashboard-gradient-card border border-white/40 ring-0 transition-all duration-200 hover:-translate-y-0.5 hover:shadow-md dark:border-white/10"
			>
				<Card.Header class="flex flex-row items-center justify-between gap-3">
					<div class="space-y-1">
						<Card.Description>{metric.label}</Card.Description>
						<Card.Title class="text-2xl">{metric.value}</Card.Title>
					</div>
					<div
						class={`flex size-10 shrink-0 items-center justify-center rounded-full ${metric.tone}`}
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
