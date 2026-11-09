<script lang="ts">
	import { replaceState } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import { onDestroy, onMount } from 'svelte';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import {
		listPeriods,
		profitSharingPeriodStatuses,
		type ProfitSharingPeriod,
		type ProfitSharingPeriodStatus
	} from '$lib/api/profit-sharing.js';
	import { ApiError } from '$lib/api/types.js';
	import DataPagination, { type PaginationState } from '$lib/components/data-pagination.svelte';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import * as Card from '$lib/components/ui/card/index.js';
	import * as Select from '$lib/components/ui/select/index.js';
	import SearchField from '$lib/components/search-field.svelte';
	import { Button } from '$lib/components/ui/button/index.js';
	import PeriodConfigDialog from './components/period-config-dialog.svelte';
	import PeriodsTable from './components/periods-table.svelte';
	import { t } from '$lib/i18n/locale.svelte.js';

	const defaultQuery = {
		page: 1,
		limit: 20,
		search: '',
		status: 'all'
	};

	let periods = $state<ProfitSharingPeriod[]>([]);
	const pageSizeOptions = [10, 20, 50, 100];
	let pagination = $state<PaginationState>({
		page: 1,
		pageSize: 20,
		totalRecords: 0,
		totalPages: 0
	});
	let isLoading = $state(true);
	let errorMessage = $state<string | null>(null);
	const initialQuery = parseQuery(page.url.searchParams);
	let query = $state(initialQuery);
	let searchValue = $state(initialQuery.search);
	let statusFilter = $state(initialQuery.status);
	let isConfigOpen = $state(false);
	let editingPeriod = $state<ProfitSharingPeriod | null>(null);
	let latestRequestId = 0;
	let searchTimer: ReturnType<typeof setTimeout> | undefined;

	onMount(() => {
		void loadPeriods(query);
	});

	onDestroy(() => {
		if (searchTimer) clearTimeout(searchTimer);
	});

	async function loadPeriods(params: typeof query) {
		const requestId = ++latestRequestId;
		isLoading = true;
		errorMessage = null;

		try {
			const result = await listPeriods({
				page: params.page,
				limit: params.limit,
				search: params.search || undefined,
				status: params.status === 'all' ? undefined : [params.status as ProfitSharingPeriodStatus]
			});
			if (requestId !== latestRequestId) return;
			periods = result.periods;
			pagination = result.pagination;
		} catch (error) {
			if (requestId !== latestRequestId) return;
			periods = [];
			errorMessage = getErrorMessage(error);
		} finally {
			if (requestId === latestRequestId) {
				isLoading = false;
			}
		}
	}

	function updateSearchValue(value: string) {
		searchValue = value;
		if (searchTimer) clearTimeout(searchTimer);

		searchTimer = setTimeout(() => {
			query = { ...query, page: 1, search: value.trim() };
			syncQueryUrl(query);
			void loadPeriods(query);
		}, 350);
	}

	function updateStatusFilter(status: string) {
		if (
			status !== 'all' &&
			!profitSharingPeriodStatuses.includes(status as ProfitSharingPeriodStatus)
		)
			return;
		statusFilter = status;
		query = { ...query, page: 1, status };
		syncQueryUrl(query);
		void loadPeriods(query);
	}

	function updatePage(nextPage: number) {
		query = { ...query, page: nextPage };
		syncQueryUrl(query);
		void loadPeriods(query);
	}

	function updatePageSize(nextSize: number) {
		query = { ...query, page: 1, limit: nextSize };
		syncQueryUrl(query);
		void loadPeriods(query);
	}

	function openCreateDialog() {
		editingPeriod = null;
		isConfigOpen = true;
	}

	function openEditDialog(period: ProfitSharingPeriod) {
		editingPeriod = period;
		isConfigOpen = true;
	}

	async function handleSaved() {
		await loadPeriods(query);
	}

	function parseQuery(searchParams: URLSearchParams) {
		const statusParam = searchParams.get('status') ?? 'all';
		return {
			...defaultQuery,
			page: Math.max(1, Number(searchParams.get('page')) || 1),
			limit: Math.min(100, Math.max(1, Number(searchParams.get('limit')) || 20)),
			search: searchParams.get('search') ?? '',
			status: profitSharingPeriodStatuses.includes(statusParam as ProfitSharingPeriodStatus)
				? statusParam
				: 'all'
		};
	}

	function syncQueryUrl(nextQuery: typeof query) {
		const nextUrl = new URL(page.url);
		nextUrl.searchParams.set('page', String(nextQuery.page));
		nextUrl.searchParams.set('limit', String(nextQuery.limit));

		if (nextQuery.search) {
			nextUrl.searchParams.set('search', nextQuery.search);
		} else {
			nextUrl.searchParams.delete('search');
		}

		if (nextQuery.status === 'all') {
			nextUrl.searchParams.delete('status');
		} else {
			nextUrl.searchParams.set('status', nextQuery.status);
		}

		replaceState(resolve(`${nextUrl.pathname}${nextUrl.search}` as '/'), page.state);
	}

	function getErrorMessage(error: unknown) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return t('finances.profitSharing.errorLoad');
	}
</script>

<svelte:head>
	<title>{t('finances.profitSharing.listTitle')} | PointFlow</title>
</svelte:head>

<div class="flex flex-col gap-6">
	<Card.Root>
		<Card.Header>
			<div>
				<Card.Title>{t('finances.profitSharing.listTitle')}</Card.Title>
				<Card.Description>{t('finances.profitSharing.listSubtitle')}</Card.Description>
			</div>
			<Card.Action>
				<Button class="w-full sm:w-auto" onclick={openCreateDialog}>
					<PlusIcon />
					{t('finances.profitSharing.addBtn')}
				</Button>
			</Card.Action>
		</Card.Header>

		<Card.Content class="flex flex-col gap-3">
			<div class="grid gap-3 sm:grid-cols-[minmax(0,1fr)_220px]">
				<SearchField
					value={searchValue}
					disabled={isLoading}
					ariaLabel={t('finances.profitSharing.searchAriaLabel')}
					placeholder={t('finances.profitSharing.searchPlaceholder')}
					onValueChange={updateSearchValue}
				/>
				<Select.Root
					type="single"
					value={statusFilter}
					disabled={isLoading}
					onValueChange={(value) => {
						if (value) updateStatusFilter(value);
					}}
				>
					<Select.Trigger class="w-full" aria-label={t('finances.profitSharing.statusFilter')}>
						<Select.Value placeholder={t('finances.profitSharing.statusAll')} />
					</Select.Trigger>
					<Select.Content>
						<Select.Item value="all" label={t('finances.profitSharing.statusAll')}
							>{t('finances.profitSharing.statusAll')}</Select.Item
						>
						<Select.Item value="draft" label={t('finances.profitSharing.statusDraft')}
							>{t('finances.profitSharing.statusDraft')}</Select.Item
						>
						<Select.Item value="calculated" label={t('finances.profitSharing.statusCalculated')}
							>{t('finances.profitSharing.statusCalculated')}</Select.Item
						>
						<Select.Item value="finalized" label={t('finances.profitSharing.statusFinalized')}
							>{t('finances.profitSharing.statusFinalized')}</Select.Item
						>
						<Select.Item value="cancelled" label={t('finances.profitSharing.statusCancelled')}
							>{t('finances.profitSharing.statusCancelled')}</Select.Item
						>
					</Select.Content>
				</Select.Root>
			</div>

			{#if errorMessage && !isLoading}
				<ErrorAlert title={t('finances.profitSharing.errorLoad')} message={errorMessage} />
			{:else}
				<PeriodsTable
					{periods}
					{isLoading}
					onEdit={openEditDialog}
					emptyMessage={query.search || query.status !== 'all'
						? t('finances.profitSharing.emptySearch')
						: t('finances.profitSharing.emptyDefault')}
				/>
			{/if}
		</Card.Content>
		{#if !isLoading && !errorMessage}
			<Card.Footer>
				<DataPagination
					{pagination}
					{pageSizeOptions}
					onPageChange={updatePage}
					onPageSizeChange={updatePageSize}
				/>
			</Card.Footer>
		{/if}
	</Card.Root>

	<PeriodConfigDialog bind:open={isConfigOpen} period={editingPeriod} onSaved={handleSaved} />
</div>
