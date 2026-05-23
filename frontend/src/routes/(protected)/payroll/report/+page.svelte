<script lang="ts">
	import { replaceState } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import { onMount } from 'svelte';
	import CalendarIcon from '@lucide/svelte/icons/calendar';
	import XIcon from '@lucide/svelte/icons/x';
	import { parseDate, type DateValue } from '@internationalized/date';
	import type { DateRange } from 'bits-ui';
	import {
		listPayrollReport,
		type PayrollPagination,
		type PayrollReport,
		type PayrollReportListParams
	} from '$lib/api/payroll.js';
	import { ApiError } from '$lib/api/types.js';
	import DataPagination from '$lib/components/data-pagination.svelte';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import PageHeader from '$lib/components/page-header.svelte';
	import { Button, buttonVariants } from '$lib/components/ui/button/index.js';
	import * as Popover from '$lib/components/ui/popover/index.js';
	import RangeCalendar from '$lib/components/ui/range-calendar/range-calendar.svelte';
	import { formatDate } from '$lib/formatters.js';
	import { cn } from '$lib/utils.js';
	import PayrollReportTable from './components/payroll-report-table.svelte';

	const pageSizeOptions = [10, 25, 50, 100];
	const defaultPagination: PayrollPagination = {
		page: 1,
		pageSize: 10,
		totalRecords: 0,
		totalPages: 0
	};

	let reports = $state<PayrollReport[]>([]);
	let pagination = $state<PayrollPagination>(defaultPagination);
	let isLoading = $state(true);
	let errorMessage = $state<string | null>(null);
	const initialQuery = parseQuery(page.url.searchParams);
	let query = $state<Required<PayrollReportListParams>>(initialQuery);
	let dateRangeValue = $state<DateRange>({
		start: dateStringToDateValue(initialQuery.from),
		end: dateStringToDateValue(initialQuery.to)
	});
	let isDateRangeOpen = $state(false);
	let latestRequestId = 0;

	onMount(() => {
		void loadPayrollReport(query);
	});

	$effect(() => {
		const from = dateValueToDateString(dateRangeValue.start);
		const to = dateValueToDateString(dateRangeValue.end);
		if (from === query.from && to === query.to) return;

		updateDateRangeFilter(from, to);
	});

	async function loadPayrollReport(params: PayrollReportListParams) {
		const requestId = ++latestRequestId;
		isLoading = true;
		errorMessage = null;

		try {
			const result = await listPayrollReport(params);
			if (requestId !== latestRequestId) return;

			if (
				result.pagination.totalPages > 0 &&
				result.pagination.page > result.pagination.totalPages
			) {
				query = { ...query, page: result.pagination.totalPages };
				syncQueryUrl(query);
				await loadPayrollReport(query);
				return;
			}

			reports = result.reports;
			pagination = result.pagination;
		} catch (error) {
			if (requestId !== latestRequestId) return;
			errorMessage = getErrorMessage(error);
			reports = [];
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
		void loadPayrollReport(query);
	}

	function updatePage(nextPage: number) {
		const page = Math.max(1, Math.trunc(nextPage));
		if (page === query.page) return;

		query = { ...query, page };
		syncQueryUrl(query);
		void loadPayrollReport(query);
	}

	function updateDateRangeFilter(from: string, to: string) {
		query = { ...query, from, to, page: 1 };
		syncQueryUrl(query);
		void loadPayrollReport(query);
	}

	function clearDateRangeFilter() {
		dateRangeValue = { start: undefined, end: undefined };
		isDateRangeOpen = false;
	}

	function parseQuery(searchParams: URLSearchParams): Required<PayrollReportListParams> {
		const page = Number(searchParams.get('page') ?? defaultPagination.page);
		const limit = Number(searchParams.get('limit') ?? defaultPagination.pageSize);
		const from = searchParams.get('from') ?? '';
		const to = searchParams.get('to') ?? '';

		return {
			page: Number.isFinite(page) ? Math.max(1, Math.trunc(page)) : defaultPagination.page,
			limit: pageSizeOptions.includes(limit) ? limit : defaultPagination.pageSize,
			from,
			to
		};
	}

	function syncQueryUrl(nextQuery: Required<PayrollReportListParams>) {
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

		setOptionalSearchParam(nextUrl, 'from', nextQuery.from);
		setOptionalSearchParam(nextUrl, 'to', nextQuery.to);

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

	function getErrorMessage(error: unknown) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return 'Failed to load payroll report.';
	}
</script>

<svelte:head>
	<title>Payroll Report | POS System</title>
</svelte:head>

<div class="flex flex-col gap-6">
	<PageHeader title="Payroll Report" description="Review payroll totals by payroll slip period." />

	<section class="flex flex-col gap-3" aria-labelledby="payroll-report-table-title">
		<h2 id="payroll-report-table-title" class="sr-only">Payroll report records</h2>

		<div class="flex justify-start sm:justify-end">
			<Popover.Root bind:open={isDateRangeOpen}>
				<Popover.Trigger
					class={cn(
						buttonVariants({ variant: 'outline' }),
						'h-9 w-full justify-start gap-2 px-3 text-left font-normal sm:w-[280px]',
						!query.from && !query.to && 'text-muted-foreground'
					)}
					disabled={isLoading}
					aria-label="Filter payroll report by date range"
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
			<ErrorAlert title="Failed to load payroll report." message={errorMessage} />
		{:else}
			<PayrollReportTable
				{reports}
				{isLoading}
				emptyMessage={query.from || query.to
					? 'No payroll report rows match your filters.'
					: 'No payroll report rows yet.'}
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
