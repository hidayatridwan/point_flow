<script lang="ts">
	import { replaceState } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import { onDestroy, onMount } from 'svelte';
	import CalendarIcon from '@lucide/svelte/icons/calendar';
	import CircleCheckIcon from '@lucide/svelte/icons/circle-check';
	import CircleIcon from '@lucide/svelte/icons/circle';
	import CircleOffIcon from '@lucide/svelte/icons/circle-off';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import XIcon from '@lucide/svelte/icons/x';
	import { parseDate, type DateValue } from '@internationalized/date';
	import type { DateRange } from 'bits-ui';
	import {
		listStockAdjustments,
		stockAdjustmentStatuses,
		type StockAdjustment,
		type StockAdjustmentListParams,
		type StockAdjustmentPagination,
		type StockAdjustmentStatus
	} from '$lib/api/stock-adjustments.js';
	import { ApiError } from '$lib/api/types.js';
	import DataTableFacetedFilter from '$lib/components/data-table-faceted-filter.svelte';
	import DataPagination from '$lib/components/data-pagination.svelte';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import PageHeader from '$lib/components/page-header.svelte';
	import SearchField from '$lib/components/search-field.svelte';
	import { Button, buttonVariants } from '$lib/components/ui/button/index.js';
	import * as Popover from '$lib/components/ui/popover/index.js';
	import RangeCalendar from '$lib/components/ui/range-calendar/range-calendar.svelte';
	import { formatDate } from '$lib/formatters.js';
	import { cn } from '$lib/utils.js';
	import StockAdjustmentCreateDialog from './components/stock-adjustment-create-dialog.svelte';
	import StockAdjustmentsTable from './components/stock-adjustments-table.svelte';
	import { t } from '$lib/i18n/locale.svelte.js';

	const pageSizeOptions = [10, 25, 50, 100];
	const defaultPagination: StockAdjustmentPagination = {
		page: 1,
		pageSize: 10,
		totalRecords: 0,
		totalPages: 0
	};
	const statusFilterOptions = [
		{ value: 'draft', label: 'Draft', icon: CircleIcon },
		{ value: 'posted', label: 'Posted', icon: CircleCheckIcon },
		{ value: 'cancelled', label: 'Cancelled', icon: CircleOffIcon }
	];
	const statusFilterValues = new Set(statusFilterOptions.map((option) => option.value));

	let stockAdjustments = $state<StockAdjustment[]>([]);
	let pagination = $state<StockAdjustmentPagination>(defaultPagination);
	let isLoading = $state(true);
	let errorMessage = $state<string | null>(null);
	const initialQuery = parseQuery(page.url.searchParams);
	let query = $state<Required<StockAdjustmentListParams>>(initialQuery);
	let searchValue = $state(initialQuery.search);
	let dateRangeValue = $state<DateRange>({
		start: dateStringToDateValue(initialQuery.from),
		end: dateStringToDateValue(initialQuery.to)
	});
	let isDateRangeOpen = $state(false);
	let isCreateOpen = $state(false);
	let latestRequestId = 0;
	let searchTimer: ReturnType<typeof setTimeout> | undefined;

	onMount(() => {
		void loadStockAdjustments(query);
	});

	onDestroy(() => {
		if (searchTimer) clearTimeout(searchTimer);
	});

	$effect(() => {
		const from = dateValueToDateString(dateRangeValue.start);
		const to = dateValueToDateString(dateRangeValue.end);
		if (from === query.from && to === query.to) return;

		updateDateRangeFilter(from, to);
	});

	async function loadStockAdjustments(params: StockAdjustmentListParams) {
		const requestId = ++latestRequestId;
		isLoading = true;
		errorMessage = null;

		try {
			const result = await listStockAdjustments(params);
			if (requestId !== latestRequestId) return;

			if (
				result.pagination.totalPages > 0 &&
				result.pagination.page > result.pagination.totalPages
			) {
				query = { ...query, page: result.pagination.totalPages };
				syncQueryUrl(query);
				await loadStockAdjustments(query);
				return;
			}

			stockAdjustments = result.stockAdjustments;
			pagination = result.pagination;
		} catch (error) {
			if (requestId !== latestRequestId) return;
			errorMessage = getErrorMessage(error);
			stockAdjustments = [];
		} finally {
			if (requestId === latestRequestId) {
				isLoading = false;
			}
		}
	}

	function updatePageSize(pageSize: number) {
		if (!pageSizeOptions.includes(pageSize)) return;

		query = { ...query, limit: pageSize, page: 1 };
		syncQueryUrl(query);
		void loadStockAdjustments(query);
	}

	function updatePage(nextPage: number) {
		const page = Math.max(1, Math.trunc(nextPage));
		if (page === query.page) return;

		query = { ...query, page };
		syncQueryUrl(query);
		void loadStockAdjustments(query);
	}

	function updateSearchValue(value: string) {
		searchValue = value;
		if (searchTimer) clearTimeout(searchTimer);

		searchTimer = setTimeout(() => {
			query = { ...query, search: value.trim(), page: 1 };
			syncQueryUrl(query);
			void loadStockAdjustments(query);
		}, 350);
	}

	function updateDateRangeFilter(from: string, to: string) {
		query = { ...query, from, to, page: 1 };
		syncQueryUrl(query);
		void loadStockAdjustments(query);
	}

	function updateStatusFilter(value: string[]) {
		query = { ...query, status: value.filter(isStatusFilterValue), page: 1 };
		syncQueryUrl(query);
		void loadStockAdjustments(query);
	}

	function clearDateRangeFilter() {
		dateRangeValue = { start: undefined, end: undefined };
		isDateRangeOpen = false;
	}

	async function refreshStockAdjustmentsFromFirstPage() {
		query = { ...query, page: 1 };
		syncQueryUrl(query);
		await loadStockAdjustments(query);
	}

	async function refreshStockAdjustmentsAfterStatusChange() {
		await loadStockAdjustments(query);
	}

	function openCreateDialog() {
		isCreateOpen = true;
	}

	function parseQuery(searchParams: URLSearchParams): Required<StockAdjustmentListParams> {
		const page = Number(searchParams.get('page') ?? defaultPagination.page);
		const limit = Number(searchParams.get('limit') ?? defaultPagination.pageSize);
		const search = searchParams.get('search') ?? '';
		const from = searchParams.get('from') ?? '';
		const to = searchParams.get('to') ?? '';
		const status = parseStatusQuery(searchParams.get('status') ?? '');

		return {
			page: Number.isFinite(page) ? Math.max(1, Math.trunc(page)) : defaultPagination.page,
			limit: pageSizeOptions.includes(limit) ? limit : defaultPagination.pageSize,
			search,
			from,
			to,
			status
		};
	}

	function syncQueryUrl(nextQuery: Required<StockAdjustmentListParams>) {
		const nextUrl = new URL(page.url);

		if (nextQuery.page === defaultPagination.page) {
			nextUrl.searchParams.delete('page');
		} else {
			nextUrl.searchParams.set('page', String(nextQuery.page));
		}

		if (nextQuery.limit === defaultPagination.pageSize) {
			nextUrl.searchParams.delete('limit');
		} else {
			nextUrl.searchParams.set('limit', String(nextQuery.limit));
		}

		setOptionalSearchParam(nextUrl, 'search', nextQuery.search);
		setOptionalSearchParam(nextUrl, 'from', nextQuery.from);
		setOptionalSearchParam(nextUrl, 'to', nextQuery.to);
		setOptionalSearchParam(nextUrl, 'status', nextQuery.status.join(','));

		replaceState(resolve(`${nextUrl.pathname}${nextUrl.search}` as '/'), page.state);
	}

	function setOptionalSearchParam(url: URL, key: string, value: string) {
		if (value) {
			url.searchParams.set(key, value);
		} else {
			url.searchParams.delete(key);
		}
	}

	function dateStringToDateValue(value: string): DateValue | undefined {
		if (!value) return undefined;

		try {
			return parseDate(value);
		} catch {
			return undefined;
		}
	}

	function dateValueToDateString(value: DateValue | undefined) {
		return value?.toString() ?? '';
	}

	function getDateRangeLabel() {
		if (query.from && query.to) return `${formatDate(query.from)} - ${formatDate(query.to)}`;
		if (query.from) return `From ${formatDate(query.from)}`;
		if (query.to) return `Until ${formatDate(query.to)}`;
		return 'Select date range';
	}

	function parseStatusQuery(value: string): StockAdjustmentStatus[] {
		return value.split(',').filter(isStatusFilterValue);
	}

	function isStatusFilterValue(value: string): value is StockAdjustmentStatus {
		return (
			statusFilterValues.has(value) &&
			stockAdjustmentStatuses.includes(value as StockAdjustmentStatus)
		);
	}

	function getErrorMessage(error: unknown) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return t('inventory.stockAdjustments.errorLoad');
	}
</script>

<svelte:head>
	<title>Stock Adjustments | POS System</title>
</svelte:head>

<div class="flex flex-col gap-6">
	<PageHeader
		title={t('inventory.stockAdjustments.title')}
		description={t('inventory.stockAdjustments.subtitle')}
	>
		{#snippet actions()}
			<Button class="w-full sm:w-auto" onclick={openCreateDialog}>
				<PlusIcon />
				{t('inventory.stockAdjustments.addBtn')}
			</Button>
		{/snippet}
	</PageHeader>

	<section class="flex flex-col gap-3" aria-labelledby="stock-adjustments-table-title">
		<h2 id="stock-adjustments-table-title" class="sr-only">Stock adjustment records</h2>

		<div class="grid gap-3 lg:grid-cols-[minmax(0,1fr)_auto] lg:items-center">
			<div class="flex flex-col gap-2 sm:flex-row sm:items-center">
				<SearchField
					value={searchValue}
					disabled={isLoading}
					ariaLabel="Search stock adjustments"
					placeholder={t('inventory.stockAdjustments.searchPlaceholder')}
					class="sm:max-w-sm"
					onValueChange={updateSearchValue}
				/>
				<DataTableFacetedFilter
					title="Status"
					options={statusFilterOptions}
					value={query.status}
					disabled={isLoading}
					onValueChange={updateStatusFilter}
				/>
			</div>

			<Popover.Root bind:open={isDateRangeOpen}>
				<Popover.Trigger
					class={cn(
						buttonVariants({ variant: 'outline' }),
						'h-9 w-full justify-start gap-2 px-3 text-left font-normal sm:w-[280px]',
						!query.from && !query.to && 'text-muted-foreground'
					)}
					disabled={isLoading}
					aria-label="Filter stock adjustments by date range"
				>
					<CalendarIcon class="size-4" />
					<span class="truncate">{getDateRangeLabel()}</span>
				</Popover.Trigger>
				<Popover.Content align="end" class="w-auto max-w-[calc(100vw-2rem)] overflow-x-auto p-0">
					<RangeCalendar
						bind:value={dateRangeValue}
						class="rounded-md border-0 shadow-none"
						numberOfMonths={2}
					/>
					{#if query.from || query.to}
						<div class="flex justify-end border-t bg-popover p-2">
							<Button type="button" variant="ghost" size="sm" onclick={clearDateRangeFilter}>
								<XIcon />
								Clear
							</Button>
						</div>
					{/if}
				</Popover.Content>
			</Popover.Root>
		</div>

		{#if errorMessage && !isLoading}
			<ErrorAlert title={t('inventory.stockAdjustments.errorLoad')} message={errorMessage} />
		{:else}
			<StockAdjustmentsTable
				{stockAdjustments}
				{isLoading}
				onStockAdjustmentChanged={refreshStockAdjustmentsAfterStatusChange}
				emptyMessage={query.search || query.status.length || query.from || query.to
					? t('inventory.stockAdjustments.emptySearch')
					: t('inventory.stockAdjustments.emptyDefault')}
			/>
			{#if !isLoading}
				<DataPagination
					{pagination}
					{pageSizeOptions}
					onPageChange={updatePage}
					onPageSizeChange={updatePageSize}
				/>
			{/if}
		{/if}
	</section>

	<StockAdjustmentCreateDialog
		bind:open={isCreateOpen}
		onCreated={refreshStockAdjustmentsFromFirstPage}
	/>
</div>
