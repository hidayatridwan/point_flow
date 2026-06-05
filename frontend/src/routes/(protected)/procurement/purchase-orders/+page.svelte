<script lang="ts">
	import { replaceState } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import { onDestroy, onMount } from 'svelte';
	import CalendarIcon from '@lucide/svelte/icons/calendar';
	import CircleDotIcon from '@lucide/svelte/icons/circle-dot';
	import CircleIcon from '@lucide/svelte/icons/circle';
	import CircleOffIcon from '@lucide/svelte/icons/circle-off';
	import PackageCheckIcon from '@lucide/svelte/icons/package-check';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import SendIcon from '@lucide/svelte/icons/send';
	import XIcon from '@lucide/svelte/icons/x';
	import { parseDate, type DateValue } from '@internationalized/date';
	import type { DateRange } from 'bits-ui';
	import {
		listPurchaseOrders,
		purchaseOrderStatuses,
		type PurchaseOrder,
		type PurchaseOrderListParams,
		type PurchaseOrderPagination,
		type PurchaseOrderStatus
	} from '$lib/api/purchase-orders.js';
	import { ApiError } from '$lib/api/types.js';
	import DataTableFacetedFilter from '$lib/components/data-table-faceted-filter.svelte';
	import DataPagination from '$lib/components/data-pagination.svelte';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import * as Card from '$lib/components/ui/card/index.js';
	import SearchField from '$lib/components/search-field.svelte';
	import { Button, buttonVariants } from '$lib/components/ui/button/index.js';
	import * as Popover from '$lib/components/ui/popover/index.js';
	import RangeCalendar from '$lib/components/ui/range-calendar/range-calendar.svelte';
	import { formatDate } from '$lib/formatters.js';
	import { cn } from '$lib/utils.js';
	import PurchaseOrderCreateDialog from './components/purchase-order-create-dialog.svelte';
	import PurchaseOrdersTable from './components/purchase-orders-table.svelte';
	import { t } from '$lib/i18n/locale.svelte.js';

	const pageSizeOptions = [10, 25, 50, 100];
	const defaultPagination: PurchaseOrderPagination = {
		page: 1,
		pageSize: 10,
		totalRecords: 0,
		totalPages: 0
	};
	const statusFilterOptions = [
		{ value: 'draft', label: t('procurement.purchaseOrders.filterDraft'), icon: CircleIcon },
		{ value: 'ordered', label: t('procurement.purchaseOrders.filterOrdered'), icon: SendIcon },
		{
			value: 'partially_received',
			label: t('procurement.purchaseOrders.filterPartiallyReceived'),
			icon: CircleDotIcon
		},
		{
			value: 'received',
			label: t('procurement.purchaseOrders.filterReceived'),
			icon: PackageCheckIcon
		},
		{
			value: 'cancelled',
			label: t('procurement.purchaseOrders.filterCancelled'),
			icon: CircleOffIcon
		}
	];
	const statusFilterValues = new Set(statusFilterOptions.map((option) => option.value));

	let purchaseOrders = $state<PurchaseOrder[]>([]);
	let pagination = $state<PurchaseOrderPagination>(defaultPagination);
	let isLoading = $state(true);
	let errorMessage = $state<string | null>(null);
	const initialQuery = parseQuery(page.url.searchParams);
	let query = $state<Required<PurchaseOrderListParams>>(initialQuery);
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
		void loadPurchaseOrders(query);
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

	async function loadPurchaseOrders(params: PurchaseOrderListParams) {
		const requestId = ++latestRequestId;
		isLoading = true;
		errorMessage = null;

		try {
			const result = await listPurchaseOrders(params);
			if (requestId !== latestRequestId) return;

			if (
				result.pagination.totalPages > 0 &&
				result.pagination.page > result.pagination.totalPages
			) {
				query = { ...query, page: result.pagination.totalPages };
				syncQueryUrl(query);
				await loadPurchaseOrders(query);
				return;
			}

			purchaseOrders = result.purchaseOrders;
			pagination = result.pagination;
		} catch (error) {
			if (requestId !== latestRequestId) return;
			errorMessage = getErrorMessage(error);
			purchaseOrders = [];
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
		void loadPurchaseOrders(query);
	}

	function updatePage(nextPage: number) {
		const page = Math.max(1, Math.trunc(nextPage));
		if (page === query.page) return;

		query = { ...query, page };
		syncQueryUrl(query);
		void loadPurchaseOrders(query);
	}

	function updateSearchValue(value: string) {
		searchValue = value;
		if (searchTimer) clearTimeout(searchTimer);

		searchTimer = setTimeout(() => {
			query = { ...query, search: value.trim(), page: 1 };
			syncQueryUrl(query);
			void loadPurchaseOrders(query);
		}, 350);
	}

	function updateDateRangeFilter(from: string, to: string) {
		query = { ...query, from, to, page: 1 };
		syncQueryUrl(query);
		void loadPurchaseOrders(query);
	}

	function updateStatusFilter(value: string[]) {
		query = { ...query, status: value.filter(isStatusFilterValue), page: 1 };
		syncQueryUrl(query);
		void loadPurchaseOrders(query);
	}

	function clearDateRangeFilter() {
		dateRangeValue = { start: undefined, end: undefined };
		isDateRangeOpen = false;
	}

	function updatePurchaseOrderInList(updatedPurchaseOrder: PurchaseOrder) {
		purchaseOrders = purchaseOrders.map((purchaseOrder) =>
			purchaseOrder.id === updatedPurchaseOrder.id ? updatedPurchaseOrder : purchaseOrder
		);
	}

	async function refreshPurchaseOrdersAfterCreate() {
		query = { ...query, page: 1 };
		syncQueryUrl(query);
		await loadPurchaseOrders(query);
	}

	function openCreateDialog() {
		isCreateOpen = true;
	}

	function parseQuery(searchParams: URLSearchParams): Required<PurchaseOrderListParams> {
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

	function syncQueryUrl(nextQuery: Required<PurchaseOrderListParams>) {
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

	function parseStatusQuery(value: string): PurchaseOrderStatus[] {
		return value.split(',').filter(isStatusFilterValue);
	}

	function isStatusFilterValue(value: string): value is PurchaseOrderStatus {
		return (
			statusFilterValues.has(value) && purchaseOrderStatuses.includes(value as PurchaseOrderStatus)
		);
	}

	function getErrorMessage(error: unknown) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return t('procurement.purchaseOrders.errorLoad');
	}
</script>

<svelte:head>
	<title>Purchase Orders | PointFlow</title>
</svelte:head>

<div class="flex flex-col gap-6">
	<Card.Root>
		<Card.Header>
			<div>
				<Card.Title>{t('procurement.purchaseOrders.title')}</Card.Title>
				<Card.Description>{t('procurement.purchaseOrders.subtitle')}</Card.Description>
			</div>
			<Card.Action>
				<Button class="w-full sm:w-auto" onclick={openCreateDialog}>
					<PlusIcon />
					{t('procurement.purchaseOrders.addBtn')}
				</Button>
			</Card.Action>
		</Card.Header>

		<Card.Content class="flex flex-col gap-3">
			<div class="grid gap-3 lg:grid-cols-[minmax(0,1fr)_auto] lg:items-center">
				<div class="flex flex-col gap-2 sm:flex-row sm:items-center">
					<SearchField
						value={searchValue}
						disabled={isLoading}
						ariaLabel="Search purchase orders"
						placeholder={t('procurement.purchaseOrders.searchPlaceholder')}
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
							'h-9 w-full justify-start gap-2 px-3 text-left font-normal sm:w-70',
							!query.from && !query.to && 'text-muted-foreground'
						)}
						disabled={isLoading}
						aria-label="Filter purchase orders by date range"
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
				<ErrorAlert title={t('procurement.purchaseOrders.errorLoad')} message={errorMessage} />
			{:else}
				<PurchaseOrdersTable
					{purchaseOrders}
					{isLoading}
					onPurchaseOrderChanged={updatePurchaseOrderInList}
					emptyMessage={query.search || query.status.length || query.from || query.to
						? t('procurement.purchaseOrders.emptySearch')
						: t('procurement.purchaseOrders.emptyDefault')}
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

	<PurchaseOrderCreateDialog
		bind:open={isCreateOpen}
		onCreated={refreshPurchaseOrdersAfterCreate}
	/>
</div>
