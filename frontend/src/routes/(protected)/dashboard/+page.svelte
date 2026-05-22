<script lang="ts">
	import { onMount } from 'svelte';
	import BanknoteIcon from '@lucide/svelte/icons/banknote';
	import ClipboardListIcon from '@lucide/svelte/icons/clipboard-list';
	import PackageSearchIcon from '@lucide/svelte/icons/package-search';
	import ReceiptTextIcon from '@lucide/svelte/icons/receipt-text';
	import TruckIcon from '@lucide/svelte/icons/truck';
	import UsersIcon from '@lucide/svelte/icons/users';
	import { getDashboardSummary, type DashboardSummary } from '$lib/api/dashboard.js';
	import { ApiError } from '$lib/api/types.js';
	import SessionTransition from '$lib/components/session-transition.svelte';
	import * as Alert from '$lib/components/ui/alert/index.js';
	import * as Card from '$lib/components/ui/card/index.js';
	import CircleAlertIcon from '@lucide/svelte/icons/circle-alert';
	import type { Component } from 'svelte';

	type Metric = {
		label: string;
		value: string;
		description: string;
		icon: Component;
		tone: string;
	};

	let summary = $state<DashboardSummary | null>(null);
	let isLoading = $state(true);
	let errorMessage = $state<string | null>(null);

	const metrics = $derived(summary ? buildMetrics(summary) : []);

	onMount(() => {
		void loadDashboard();
	});

	async function loadDashboard() {
		isLoading = true;
		errorMessage = null;

		try {
			summary = await getDashboardSummary();
		} catch (error) {
			errorMessage = getErrorMessage(error);
			summary = null;
		} finally {
			isLoading = false;
		}
	}

	function buildMetrics(data: DashboardSummary): Metric[] {
		return [
			{
				label: 'Revenue Today',
				value: formatCurrency(data.revenueToday),
				description: 'Total paid sales recorded today.',
				icon: BanknoteIcon,
				tone: 'border-emerald-200 bg-emerald-50 text-emerald-700'
			},
			{
				label: 'Transactions Today',
				value: formatNumber(data.transactionCountToday),
				description: 'Completed sales transactions.',
				icon: ReceiptTextIcon,
				tone: 'border-sky-200 bg-sky-50 text-sky-700'
			},
			{
				label: 'Low Stock Products',
				value: formatNumber(data.lowStockProductCount),
				description: 'SKUs at or below minimum stock.',
				icon: PackageSearchIcon,
				tone: 'border-amber-200 bg-amber-50 text-amber-700'
			},
			{
				label: 'Pending Purchase Orders',
				value: formatNumber(data.pendingPurchaseOrderCount),
				description: 'Orders waiting for receiving.',
				icon: TruckIcon,
				tone: 'border-violet-200 bg-violet-50 text-violet-700'
			},
			{
				label: 'Draft Adjustments',
				value: formatNumber(data.draftStockAdjustmentCount),
				description: 'Stock adjustments not posted yet.',
				icon: ClipboardListIcon,
				tone: 'border-rose-200 bg-rose-50 text-rose-700'
			},
			{
				label: 'Active Employees',
				value: formatNumber(data.activeEmployeeCount),
				description: 'Employees available for operations.',
				icon: UsersIcon,
				tone: 'border-cyan-200 bg-cyan-50 text-cyan-700'
			}
		];
	}

	function formatCurrency(value: number) {
		return new Intl.NumberFormat('id-ID', {
			style: 'currency',
			currency: 'IDR',
			maximumFractionDigits: 0
		}).format(value);
	}

	function formatNumber(value: number) {
		return new Intl.NumberFormat('id-ID').format(value);
	}

	function getErrorMessage(error: unknown) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return 'Failed to load dashboard summary.';
	}
</script>

<svelte:head>
	<title>Dashboard | POS System</title>
</svelte:head>

<div class="flex flex-col gap-6">
	<div class="flex flex-col gap-1">
		<h1 class="text-xl font-semibold tracking-tight">Dashboard</h1>
		<p class="max-w-2xl text-sm text-muted-foreground">
			Store operations summary for sales, stock, procurement, and payroll.
		</p>
	</div>

	{#if isLoading}
		<SessionTransition title="Loading dashboard..." description="Please wait a moment." />
	{:else if errorMessage}
		<Alert.Root variant="destructive">
			<CircleAlertIcon />
			<Alert.Title>Failed to load dashboard.</Alert.Title>
			<Alert.Description>{errorMessage}</Alert.Description>
		</Alert.Root>
	{:else}
		<section class="grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
			{#each metrics as metric (metric.label)}
				{@const Icon = metric.icon}
				<Card.Root size="sm">
					<Card.Header class="flex flex-row items-center justify-between gap-3">
						<div class="space-y-1">
							<Card.Description>{metric.label}</Card.Description>
							<Card.Title class="text-2xl">{metric.value}</Card.Title>
						</div>
						<div
							class={`flex size-10 shrink-0 items-center justify-center rounded-md border ${metric.tone}`}
						>
							<Icon class="size-5" />
						</div>
					</Card.Header>
					<Card.Content>
						<p class="text-sm text-muted-foreground">{metric.description}</p>
					</Card.Content>
				</Card.Root>
			{/each}
		</section>
	{/if}
</div>
