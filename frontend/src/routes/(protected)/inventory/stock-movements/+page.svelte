<script lang="ts">
	import { replaceState } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import { onDestroy, onMount } from 'svelte';
	import CalendarIcon from '@lucide/svelte/icons/calendar';
	import XIcon from '@lucide/svelte/icons/x';
	import { parseDate, type DateValue } from '@internationalized/date';
	import type { DateRange } from 'bits-ui';
	import {
		listStockMovements,
		stockMovementTypes,
		type StockMovement,
		type StockMovementListParams,
		type StockMovementPagination,
		type StockMovementType
	} from '$lib/api/stock-movements.js';
	import { ApiError } from '$lib/api/types.js';
	import DataPagination from '$lib/components/data-pagination.svelte';
	import DataTableFacetedFilter from '$lib/components/data-table-faceted-filter.svelte';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import * as Card from '$lib/components/ui/card/index.js';
	import SearchField from '$lib/components/search-field.svelte';
	import { Button, buttonVariants } from '$lib/components/ui/button/index.js';
	import * as Popover from '$lib/components/ui/popover/index.js';
	import RangeCalendar from '$lib/components/ui/range-calendar/range-calendar.svelte';
	import { formatDate } from '$lib/formatters.js';
	import { cn } from '$lib/utils.js';
	import StockMovementsTable from './components/stock-movements-table.svelte';
	import { t } from '$lib/i18n/locale.svelte.js';

	const pageSizeOptions = [10, 25, 50, 100];
	const defaultPagination: StockMovementPagination = {
		page: 1,
		pageSize: 10,
		totalRecords: 0,
		totalPages: 0
	};
	const typeFilterOptions = [
		{ value: 'purchase_receipt', label: 'Purchase Receipt' },
		{ value: 'adjustment', label: 'Adjustment' },
		{ value: 'sale', label: 'Sale' },
		{ value: 'conversion_in', label: 'Conversion In' },
		{ value: 'conversion_out', label: 'Conversion Out' },
		{ value: 'stock_in', label: 'Stock In' },
		{ value: 'stock_out', label: 'Stock Out' },
		{ value: 'correction', label: 'Correction' }
	];
	const typeFilterValues = new Set(typeFilterOptions.map((option) => option.value));

	let stockMovements = $state<StockMovement[]>([]);
	let pagination = $state<StockMovementPagination>(defaultPagination);
	let isLoading = $state(true);
	let errorMessage = $state<string | null>(null);
	const initialQuery = parseQuery(page.url.searchParams);
	let query = $state<Required<Omit<StockMovementListParams, 'productSkuId'>>>(initialQuery);
	let searchValue = $state(initialQuery.search);
	let dateRangeValue = $state<DateRange>({
		start: dateStringToDateValue(initialQuery.from),
		end: dateStringToDateValue(initialQuery.to)
	});
	let isDateRangeOpen = $state(false);
	let latestRequestId = 0;
	let searchTimer: ReturnType<typeof setTimeout> | undefined;

	onMount(() => {
		void loadStockMovements(query);
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

	async function loadStockMovements(params: StockMovementListParams) {
		const requestId = ++latestRequestId;
		isLoading = true;
		errorMessage = null;

		try {
			const result = await listStockMovements(params);
			if (requestId !== latestRequestId) return;

			if (
				result.pagination.totalPages > 0 &&
				result.pagination.page > result.pagination.totalPages
			) {
				query = { ...query, page: result.pagination.totalPages };
				syncQueryUrl(query);
				await loadStockMovements(query);
				return;
			}

			stockMovements = result.stockMovements;
			pagination = result.pagination;
		} catch (error) {
			if (requestId !== latestRequestId) return;
			errorMessage = getErrorMessage(error);
			stockMovements = [];
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
		void loadStockMovements(query);
	}

	function updatePage(nextPage: number) {
		const page = Math.max(1, Math.trunc(nextPage));
		if (page === query.page) return;

		query = { ...query, page };
		syncQueryUrl(query);
		void loadStockMovements(query);
	}

	function updateSearchValue(value: string) {
		searchValue = value;
		if (searchTimer) clearTimeout(searchTimer);

		searchTimer = setTimeout(() => {
			query = { ...query, search: value.trim(), page: 1 };
			syncQueryUrl(query);
			void loadStockMovements(query);
		}, 350);
	}

	function updateDateRangeFilter(from: string, to: string) {
		query = { ...query, from, to, page: 1 };
		syncQueryUrl(query);
		void loadStockMovements(query);
	}

	function updateTypeFilter(value: string[]) {
		query = { ...query, movementType: value.filter(isTypeFilterValue), page: 1 };
		syncQueryUrl(query);
		void loadStockMovements(query);
	}

	function clearDateRangeFilter() {
		dateRangeValue = { start: undefined, end: undefined };
		isDateRangeOpen = false;
	}

	function parseQuery(
		searchParams: URLSearchParams
	): Required<Omit<StockMovementListParams, 'productSkuId'>> {
		const page = Number(searchParams.get('page') ?? defaultPagination.page);
		const limit = Number(searchParams.get('limit') ?? defaultPagination.pageSize);
		const search = searchParams.get('search') ?? '';
		const from = searchParams.get('from') ?? '';
		const to = searchParams.get('to') ?? '';
		const movementType = parseTypeQuery(searchParams.get('type') ?? '');

		return {
			page: Number.isFinite(page) ? Math.max(1, Math.trunc(page)) : defaultPagination.page,
			limit: pageSizeOptions.includes(limit) ? limit : defaultPagination.pageSize,
			search,
			from,
			to,
			movementType
		};
	}

	function syncQueryUrl(nextQuery: Required<Omit<StockMovementListParams, 'productSkuId'>>) {
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
		setOptionalSearchParam(nextUrl, 'type', nextQuery.movementType.join(','));

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

	function parseTypeQuery(value: string): StockMovementType[] {
		return value.split(',').filter(isTypeFilterValue);
	}

	function isTypeFilterValue(value: string): value is StockMovementType {
		return typeFilterValues.has(value) && stockMovementTypes.includes(value as StockMovementType);
	}

	function getErrorMessage(error: unknown) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return t('inventory.stockMovements.errorLoad');
	}
</script>

<svelte:head>
	<title>Stock Movements | POS System</title>
</svelte:head>

<div class="flex flex-col gap-6">
	<Card.Root>
		<Card.Header>
			<div>
				<Card.Title>{t('inventory.stockMovements.title')}</Card.Title>
				<Card.Description>{t('inventory.stockMovements.subtitle')}</Card.Description>
			</div>
		</Card.Header>

		<Card.Content class="flex flex-col gap-3">
			<div class="grid gap-3 lg:grid-cols-[minmax(0,1fr)_auto] lg:items-center">
				<div class="flex flex-col gap-2 sm:flex-row sm:items-center">
					<SearchField
						value={searchValue}
						disabled={isLoading}
						ariaLabel={t('inventory.stockMovements.searchAriaLabel')}
						placeholder={t('inventory.stockMovements.searchPlaceholder')}
						class="sm:max-w-sm"
						onValueChange={updateSearchValue}
					/>
					<DataTableFacetedFilter
						title="Type"
						options={typeFilterOptions}
						value={query.movementType}
						disabled={isLoading}
						onValueChange={updateTypeFilter}
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
						aria-label="Filter stock movements by date range"
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
				<ErrorAlert title={t('inventory.stockMovements.errorLoad')} message={errorMessage} />
			{:else}
				<StockMovementsTable
					{stockMovements}
					{isLoading}
					emptyMessage={query.search || query.movementType.length || query.from || query.to
						? t('inventory.stockMovements.emptySearch')
						: t('inventory.stockMovements.emptyDefault')}
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
</div>
