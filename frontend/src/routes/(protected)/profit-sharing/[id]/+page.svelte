<script lang="ts">
	import { page } from '$app/state';
	import { onDestroy, onMount } from 'svelte';
	import DownloadIcon from '@lucide/svelte/icons/download';
	import {
		calculatePeriod,
		exportDistributionsCsv,
		getPeriodDetail,
		listDistributions,
		type ProfitSharingDistribution,
		type ProfitSharingPeriodDetail
	} from '$lib/api/profit-sharing.js';
	import { ApiError } from '$lib/api/types.js';
	import DataPagination, { type PaginationState } from '$lib/components/data-pagination.svelte';
	import DetailField from '$lib/components/detail-field.svelte';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import SearchField from '$lib/components/search-field.svelte';
	import { Button } from '$lib/components/ui/button/index.js';
	import * as Card from '$lib/components/ui/card/index.js';
	import { Spinner } from '$lib/components/ui/spinner/index.js';
	import * as Table from '$lib/components/ui/table/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import { formatDate, formatIdr } from '$lib/formatters.js';
	import { toast } from 'svelte-sonner';
	import { t } from '$lib/i18n/locale.svelte.js';
	import PeriodStatusBadge from '../components/period-status-badge.svelte';
	import DistributionsTable from './components/distributions-table.svelte';
	import PeriodCancelDialog from './components/period-cancel-dialog.svelte';
	import PeriodFinalizeDialog from './components/period-finalize-dialog.svelte';

	const BUCKET_LABEL_KEYS: Record<string, string> = {
		member: 'finances.profitSharing.bucketMember',
		reserve: 'finances.profitSharing.bucketReserve',
		management: 'finances.profitSharing.bucketManagement',
		supervisor: 'finances.profitSharing.bucketSupervisor',
		social: 'finances.profitSharing.bucketSocial',
		education: 'finances.profitSharing.bucketEducation'
	};

	const periodId = $derived(Number(page.params.id));

	let period = $state<ProfitSharingPeriodDetail | null>(null);
	const canAct = $derived(period?.status === 'draft' || period?.status === 'calculated');
	let isLoadingDetail = $state(true);
	let detailError = $state<string | null>(null);
	let distributions = $state<ProfitSharingDistribution[]>([]);
	let distPagination = $state<PaginationState>({
		page: 1,
		pageSize: 20,
		totalRecords: 0,
		totalPages: 0
	});
	let isLoadingDist = $state(true);
	let distError = $state<string | null>(null);
	let distSearch = $state('');
	let isCalculating = $state(false);
	let actionError = $state<string | null>(null);
	let isCancelOpen = $state(false);
	let isFinalizeOpen = $state(false);
	let isExporting = $state(false);
	let latestRequestId = 0;
	let searchTimer: ReturnType<typeof setTimeout> | undefined;

	onMount(() => {
		void loadDetail();
	});

	onDestroy(() => {
		if (searchTimer) clearTimeout(searchTimer);
	});

	async function loadDetail() {
		const requestId = ++latestRequestId;
		isLoadingDetail = true;
		detailError = null;

		try {
			const result = await getPeriodDetail(periodId);
			if (requestId !== latestRequestId) return;
			period = result;
			await loadDistributions(1, distSearch, requestId);
		} catch (error) {
			if (requestId !== latestRequestId) return;
			period = null;
			detailError = getDetailError(error);
		} finally {
			if (requestId === latestRequestId) {
				isLoadingDetail = false;
			}
		}
	}

	async function loadDistributions(pageNumber: number, search: string, parentRequest?: number) {
		isLoadingDist = true;
		distError = null;

		try {
			const result = await listDistributions(periodId, {
				page: pageNumber,
				limit: distPagination.pageSize,
				search: search || undefined
			});
			if (parentRequest !== undefined && parentRequest !== latestRequestId) return;
			distributions = result.distributions;
			distPagination = result.pagination;
		} catch (error) {
			distributions = [];
			distError = getDetailError(error);
		} finally {
			isLoadingDist = false;
		}
	}

	function updateDistSearch(value: string) {
		distSearch = value;
		if (searchTimer) clearTimeout(searchTimer);
		searchTimer = setTimeout(() => {
			void loadDistributions(1, value.trim());
		}, 350);
	}

	async function handleCalculate() {
		if (isCalculating) return;
		isCalculating = true;
		actionError = null;

		try {
			period = await calculatePeriod(periodId);
			await loadDistributions(1, distSearch);
			toast.success(t('finances.profitSharing.calculatedToast'));
		} catch (error) {
			actionError = getActionError(error, t('finances.profitSharing.errorCalculate'));
		} finally {
			isCalculating = false;
		}
	}

	async function handleExport() {
		if (isExporting) return;
		isExporting = true;
		actionError = null;

		try {
			const blob = await exportDistributionsCsv(periodId);
			const url = URL.createObjectURL(blob);
			const anchor = document.createElement('a');
			anchor.href = url;
			anchor.download = `profit-sharing-${periodId}-distributions.csv`;
			document.body.appendChild(anchor);
			anchor.click();
			anchor.remove();
			URL.revokeObjectURL(url);
			toast.success(t('finances.profitSharing.exportedToast'));
		} catch (error) {
			actionError = getActionError(error, t('finances.profitSharing.errorExport'));
		} finally {
			isExporting = false;
		}
	}

	function getDetailError(error: unknown) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return t('finances.profitSharing.errorLoadDetail');
	}

	function getActionError(error: unknown, fallback: string) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return fallback;
	}
</script>

<svelte:head>
	<title>{period?.period_name ?? t('finances.profitSharing.title')} | PointFlow</title>
</svelte:head>

<div class="flex flex-col gap-6">
	{#if isLoadingDetail}
		<Card.Root>
			<Card.Header>
				<Skeleton class="h-7 w-56" />
				<Skeleton class="h-4 w-72" />
			</Card.Header>
			<Card.Content>
				<div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
					{#each [...Array(4).keys()] as index (index)}
						<div class="space-y-2 rounded-md border p-4">
							<Skeleton class="h-4 w-24" />
							<Skeleton class="h-7 w-32" />
						</div>
					{/each}
				</div>
			</Card.Content>
		</Card.Root>
	{:else if detailError || !period}
		<ErrorAlert
			title={t('finances.profitSharing.errorLoadDetail')}
			message={detailError ?? t('finances.profitSharing.errorLoadDetail')}
		/>
	{:else}
		<Card.Root>
			<Card.Header>
				<div>
					<Card.Title>
						<span class="mr-2">{period.period_name}</span>
						<PeriodStatusBadge status={period.status} />
					</Card.Title>
					<Card.Description>{t('finances.profitSharing.detailSubtitle')}</Card.Description>
				</div>
				<Card.Action>
					<div class="flex w-full flex-wrap justify-end gap-2 sm:w-auto">
						{#if canAct}
							<Button
								variant="outline"
								disabled={isCalculating}
								onclick={() => void handleCalculate()}
							>
								{#if isCalculating}
									<Spinner />
								{/if}
								{isCalculating
									? t('finances.profitSharing.calculating')
									: period.member_count > 0
										? t('finances.profitSharing.calculateBtn')
										: t('finances.profitSharing.calculateFirstBtn')}
							</Button>
						{/if}
						{#if period.status === 'calculated'}
							<Button disabled={isCalculating} onclick={() => (isFinalizeOpen = true)}>
								{t('finances.profitSharing.finalizeBtn')}
							</Button>
						{/if}
						{#if canAct}
							<Button
								variant="destructive"
								disabled={isCalculating}
								onclick={() => (isCancelOpen = true)}
							>
								{t('finances.profitSharing.cancelBtn')}
							</Button>
						{/if}
						<Button
							variant="outline"
							disabled={isExporting || period.member_count === 0}
							onclick={() => void handleExport()}
						>
							{#if isExporting}
								<Spinner />
							{:else}
								<DownloadIcon />
							{/if}
							{isExporting
								? t('finances.profitSharing.exporting')
								: t('finances.profitSharing.exportBtn')}
						</Button>
					</div>
				</Card.Action>
			</Card.Header>

			<Card.Content class="flex flex-col gap-4">
				{#if actionError}
					<ErrorAlert title={t('common.saveFailed')} message={actionError} />
				{/if}

				<div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
					<div class="space-y-1 rounded-md border p-4">
						<p class="text-sm text-muted-foreground">{t('finances.profitSharing.poolUsed')}</p>
						<p class="text-2xl font-semibold tabular-nums">{formatIdr(period.pool_used)}</p>
						<p class="text-xs text-muted-foreground">
							{#if period.total_pool !== null && period.total_pool !== undefined}
								{t('finances.profitSharing.manualPool')}
							{:else}
								{t('finances.profitSharing.autoNetProfit')}: {formatIdr(
									period.auto_net_profit ?? 0
								)}
							{/if}
						</p>
					</div>
					<div class="space-y-1 rounded-md border p-4">
						<p class="text-sm text-muted-foreground">{t('finances.profitSharing.memberPool')}</p>
						<p class="text-2xl font-semibold tabular-nums">{formatIdr(period.member_pool)}</p>
						<p class="text-xs text-muted-foreground">{period.member_pct.toFixed(2)}%</p>
					</div>
					<div class="space-y-1 rounded-md border p-4">
						<p class="text-sm text-muted-foreground">{t('finances.profitSharing.memberCount')}</p>
						<p class="text-2xl font-semibold tabular-nums">{period.member_count}</p>
					</div>
					<div class="space-y-1 rounded-md border p-4">
						<p class="text-sm text-muted-foreground">
							{t('finances.profitSharing.totalMemberTransaction')}
						</p>
						<p class="text-2xl font-semibold tabular-nums">
							{formatIdr(period.total_member_transaction)}
						</p>
					</div>
				</div>

				<div class="grid gap-2">
					<p class="text-sm font-medium">
						{formatDate(period.start_date)} – {formatDate(period.end_date)}
					</p>
					{#if period.cancel_reason}
						<ErrorAlert
							title={t('finances.profitSharing.statusCancelled')}
							message={period.cancel_reason}
						/>
					{/if}
				</div>
			</Card.Content>
		</Card.Root>

		<Card.Root>
			<Card.Header>
				<Card.Title>{t('finances.profitSharing.allocTitle')}</Card.Title>
			</Card.Header>
			<Card.Content>
				<div class="overflow-hidden rounded-md border">
					<Table.Root>
						<Table.Header>
							<Table.Row>
								<Table.Head>{t('common.description')}</Table.Head>
								<Table.Head class="text-right">%</Table.Head>
								<Table.Head class="text-right">{t('common.total')}</Table.Head>
							</Table.Row>
						</Table.Header>
						<Table.Body>
							{#each period.allocations as allocation (allocation.key)}
								<Table.Row>
									<Table.Cell>{t(BUCKET_LABEL_KEYS[allocation.key] ?? allocation.key)}</Table.Cell>
									<Table.Cell class="text-right tabular-nums">
										{allocation.percentage.toFixed(2)}%
									</Table.Cell>
									<Table.Cell class="text-right font-medium tabular-nums">
										{formatIdr(allocation.amount)}
									</Table.Cell>
								</Table.Row>
							{/each}
						</Table.Body>
					</Table.Root>
				</div>
			</Card.Content>
		</Card.Root>

		{#if period.notes}
			<Card.Root>
				<Card.Content>
					<DetailField label={t('common.notes')} value={period.notes} />
				</Card.Content>
			</Card.Root>
		{/if}

		<Card.Root>
			<Card.Header>
				<Card.Title>{t('finances.profitSharing.distributionsTitle')}</Card.Title>
			</Card.Header>
			<Card.Content class="flex flex-col gap-3">
				<div class="grid gap-3 sm:grid-cols-[minmax(0,1fr)_220px]">
					<SearchField
						value={distSearch}
						disabled={isLoadingDist}
						ariaLabel={t('finances.profitSharing.searchAriaLabel')}
						placeholder={t('finances.profitSharing.searchPlaceholder')}
						onValueChange={updateDistSearch}
					/>
				</div>

				{#if distError && !isLoadingDist}
					<ErrorAlert title={t('finances.profitSharing.errorLoad')} message={distError} />
				{:else}
					<DistributionsTable
						{distributions}
						isLoading={isLoadingDist}
						emptyMessage={distSearch
							? t('finances.profitSharing.emptySearch')
							: t('finances.profitSharing.distributionsEmpty')}
					/>
				{/if}
			</Card.Content>
			{#if !isLoadingDist && !distError}
				<Card.Footer>
					<DataPagination
						pagination={distPagination}
						pageSizeOptions={[10, 20, 50, 100]}
						onPageChange={(nextPage) => void loadDistributions(nextPage, distSearch)}
						onPageSizeChange={() => void loadDistributions(1, distSearch)}
					/>
				</Card.Footer>
			{/if}
		</Card.Root>

		<PeriodCancelDialog bind:open={isCancelOpen} {period} onCancelled={() => void loadDetail()} />
		<PeriodFinalizeDialog
			bind:open={isFinalizeOpen}
			{period}
			onFinalized={() => void loadDetail()}
		/>
	{/if}
</div>
