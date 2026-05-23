<script lang="ts">
	import { replaceState } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import { onDestroy, onMount } from 'svelte';
	import CircleCheckIcon from '@lucide/svelte/icons/circle-check';
	import CircleOffIcon from '@lucide/svelte/icons/circle-off';
	import TriangleAlertIcon from '@lucide/svelte/icons/triangle-alert';
	import {
		listStockReport,
		stockReportStatuses,
		type StockReportItem,
		type StockReportListParams,
		type StockReportPagination,
		type StockReportStatus
	} from '$lib/api/stock-report.js';
	import { ApiError } from '$lib/api/types.js';
	import DataPagination from '$lib/components/data-pagination.svelte';
	import DataTableFacetedFilter from '$lib/components/data-table-faceted-filter.svelte';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import PageHeader from '$lib/components/page-header.svelte';
	import SearchField from '$lib/components/search-field.svelte';
	import StockReportTable from './components/stock-report-table.svelte';

	const pageSizeOptions = [10, 25, 50, 100];
	const defaultPagination: StockReportPagination = {
		page: 1,
		pageSize: 10,
		totalRecords: 0,
		totalPages: 0
	};
	const statusFilterOptions = [
		{ value: 'low_stock', label: 'Low Stock', icon: TriangleAlertIcon },
		{ value: 'in_stock', label: 'In Stock', icon: CircleCheckIcon },
		{ value: 'inactive', label: 'Inactive', icon: CircleOffIcon }
	];
	const statusFilterValues = new Set(statusFilterOptions.map((option) => option.value));

	let items = $state<StockReportItem[]>([]);
	let pagination = $state<StockReportPagination>(defaultPagination);
	let isLoading = $state(true);
	let errorMessage = $state<string | null>(null);
	const initialQuery = parseQuery(page.url.searchParams);
	let query = $state<Required<StockReportListParams>>(initialQuery);
	let searchValue = $state(initialQuery.search);
	let latestRequestId = 0;
	let searchTimer: ReturnType<typeof setTimeout> | undefined;

	onMount(() => {
		void loadStockReport(query);
	});

	onDestroy(() => {
		if (searchTimer) clearTimeout(searchTimer);
	});

	async function loadStockReport(params: StockReportListParams) {
		const requestId = ++latestRequestId;
		isLoading = true;
		errorMessage = null;

		try {
			const result = await listStockReport(params);
			if (requestId !== latestRequestId) return;

			if (
				result.pagination.totalPages > 0 &&
				result.pagination.page > result.pagination.totalPages
			) {
				query = { ...query, page: result.pagination.totalPages };
				syncQueryUrl(query);
				await loadStockReport(query);
				return;
			}

			items = result.items;
			pagination = result.pagination;
		} catch (error) {
			if (requestId !== latestRequestId) return;
			errorMessage = getErrorMessage(error);
			items = [];
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
		void loadStockReport(query);
	}

	function updatePage(nextPage: number) {
		const page = Math.max(1, Math.trunc(nextPage));
		if (page === query.page) return;

		query = { ...query, page };
		syncQueryUrl(query);
		void loadStockReport(query);
	}

	function updateSearchValue(value: string) {
		searchValue = value;
		if (searchTimer) clearTimeout(searchTimer);

		searchTimer = setTimeout(() => {
			query = { ...query, search: value.trim(), page: 1 };
			syncQueryUrl(query);
			void loadStockReport(query);
		}, 350);
	}

	function updateStatusFilter(value: string[]) {
		query = { ...query, status: value.filter(isStatusFilterValue), page: 1 };
		syncQueryUrl(query);
		void loadStockReport(query);
	}

	function parseQuery(searchParams: URLSearchParams): Required<StockReportListParams> {
		const page = Number(searchParams.get('page') ?? defaultPagination.page);
		const limit = Number(searchParams.get('limit') ?? defaultPagination.pageSize);
		const search = searchParams.get('search') ?? '';
		const status = parseStatusQuery(searchParams.get('status') ?? '');

		return {
			page: Number.isFinite(page) ? Math.max(1, Math.trunc(page)) : defaultPagination.page,
			limit: pageSizeOptions.includes(limit) ? limit : defaultPagination.pageSize,
			search,
			status
		};
	}

	function syncQueryUrl(nextQuery: Required<StockReportListParams>) {
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

	function parseStatusQuery(value: string): StockReportStatus[] {
		return value.split(',').filter(isStatusFilterValue);
	}

	function isStatusFilterValue(value: string): value is StockReportStatus {
		return (
			statusFilterValues.has(value) && stockReportStatuses.includes(value as StockReportStatus)
		);
	}

	function getErrorMessage(error: unknown) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return 'Failed to load stock report.';
	}
</script>

<svelte:head>
	<title>Stock Report | POS System</title>
</svelte:head>

<div class="flex flex-col gap-6">
	<PageHeader
		title="Stock Report"
		description="Monitor current stock, minimum stock, and on-order quantities by product SKU."
	/>

	<section class="flex flex-col gap-3" aria-labelledby="stock-report-table-title">
		<h2 id="stock-report-table-title" class="sr-only">Stock report records</h2>

		<div class="flex flex-col gap-2 sm:flex-row sm:items-center">
			<SearchField
				value={searchValue}
				disabled={isLoading}
				ariaLabel="Search stock report"
				placeholder="Search stock report..."
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

		{#if errorMessage && !isLoading}
			<ErrorAlert title="Failed to load stock report." message={errorMessage} />
		{:else}
			<StockReportTable
				{items}
				{isLoading}
				emptyMessage={query.search || query.status.length
					? 'No stock report items match your filters.'
					: 'No stock report items yet.'}
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
</div>
