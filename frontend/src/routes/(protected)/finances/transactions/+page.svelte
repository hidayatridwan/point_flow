<script lang="ts">
	import { replaceState } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import { onDestroy, onMount } from 'svelte';
	import BanknoteArrowDownIcon from '@lucide/svelte/icons/banknote-arrow-down';
	import BanknoteArrowUpIcon from '@lucide/svelte/icons/banknote-arrow-up';
	import CalendarIcon from '@lucide/svelte/icons/calendar';
	import CircleCheckIcon from '@lucide/svelte/icons/circle-check';
	import CircleOffIcon from '@lucide/svelte/icons/circle-off';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import TagsIcon from '@lucide/svelte/icons/tags';
	import XIcon from '@lucide/svelte/icons/x';
	import { parseDate, type DateValue } from '@internationalized/date';
	import type { DateRange } from 'bits-ui';
	import {
		financialTransactionStatuses,
		listFinancialCategories,
		listFinancialTransactions,
		type FinancialCategory,
		type FinancialTransaction,
		type FinancialTransactionListParams,
		type FinancialTransactionPagination,
		type FinancialTransactionStatus,
		type FinancialTransactionType
	} from '$lib/api/finances.js';
	import { ApiError } from '$lib/api/types.js';
	import DataTableFacetedFilter from '$lib/components/data-table-faceted-filter.svelte';
	import DataPagination from '$lib/components/data-pagination.svelte';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import * as Card from '$lib/components/ui/card/index.js';
	import SearchField from '$lib/components/search-field.svelte';
	import { Button, buttonVariants } from '$lib/components/ui/button/index.js';
	import * as Popover from '$lib/components/ui/popover/index.js';
	import RangeCalendar from '$lib/components/ui/range-calendar/range-calendar.svelte';
	import * as Select from '$lib/components/ui/select/index.js';
	import { formatDate } from '$lib/formatters.js';
	import { cn } from '$lib/utils.js';
	import FinancialCategoriesDialog from './components/financial-categories-dialog.svelte';
	import FinancialTransactionCreateDialog from './components/financial-transaction-create-dialog.svelte';
	import FinancialTransactionsTable from './components/financial-transactions-table.svelte';
	import { t } from '$lib/i18n/locale.svelte.js';

	type TransactionQuery = Required<
		Omit<FinancialTransactionListParams, 'type' | 'financial_category_id' | 'status'>
	> & {
		type: FinancialTransactionType | '';
		financial_category_id: number | null;
		status: FinancialTransactionStatus[];
	};

	const pageSizeOptions = [10, 25, 50, 100];
	const defaultPagination: FinancialTransactionPagination = {
		page: 1,
		pageSize: 10,
		totalRecords: 0,
		totalPages: 0
	};

	const statusFilterOptions = [
		{
			value: 'completed',
			label: t('finances.transactions.statusCompleted'),
			icon: CircleCheckIcon
		},
		{
			value: 'cancelled',
			label: t('finances.transactions.statusCancelled'),
			icon: CircleOffIcon
		}
	];

	let transactions = $state<FinancialTransaction[]>([]);
	let categories = $state<FinancialCategory[]>([]);
	let pagination = $state<FinancialTransactionPagination>(defaultPagination);
	let isLoading = $state(true);
	let errorMessage = $state<string | null>(null);
	const initialQuery = parseQuery(page.url.searchParams);
	let query = $state<TransactionQuery>(initialQuery);
	let searchValue = $state(initialQuery.search);
	let dateRangeValue = $state<DateRange>({
		start: dateStringToDateValue(initialQuery.from),
		end: dateStringToDateValue(initialQuery.to)
	});
	let isDateRangeOpen = $state(false);
	let isCreateOpen = $state(false);
	let isCategoriesOpen = $state(false);
	let latestRequestId = 0;
	let searchTimer: ReturnType<typeof setTimeout> | undefined;

	onMount(() => {
		void loadCategories();
		void loadTransactions(query);
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

	async function loadCategories() {
		try {
			const result = await listFinancialCategories({ limit: 100 });
			categories = result.categories.filter((category) => category.is_active);
		} catch {
			categories = [];
		}
	}

	async function loadTransactions(params: TransactionQuery) {
		const requestId = ++latestRequestId;
		isLoading = true;
		errorMessage = null;

		try {
			const result = await listFinancialTransactions(toListParams(params));
			if (requestId !== latestRequestId) return;

			if (
				result.pagination.totalPages > 0 &&
				result.pagination.page > result.pagination.totalPages
			) {
				query = { ...query, page: result.pagination.totalPages };
				syncQueryUrl(query);
				await loadTransactions(query);
				return;
			}

			transactions = result.transactions;
			pagination = result.pagination;
		} catch (error) {
			if (requestId !== latestRequestId) return;
			errorMessage = getErrorMessage(error);
			transactions = [];
		} finally {
			if (requestId === latestRequestId) {
				isLoading = false;
			}
		}
	}

	function toListParams(params: TransactionQuery): FinancialTransactionListParams {
		return {
			page: params.page,
			limit: params.limit,
			search: params.search,
			from: params.from,
			to: params.to,
			type: params.type || undefined,
			financial_category_id: params.financial_category_id ?? undefined,
			status: params.status
		};
	}

	function updatePageSize(pageSize: number) {
		if (!pageSizeOptions.includes(pageSize)) return;

		query = { ...query, limit: pageSize, page: 1 };
		syncQueryUrl(query);
		void loadTransactions(query);
	}

	function updatePage(nextPage: number) {
		const page = Math.max(1, Math.trunc(nextPage));
		if (page === query.page) return;

		query = { ...query, page };
		syncQueryUrl(query);
		void loadTransactions(query);
	}

	function updateSearchValue(value: string) {
		searchValue = value;
		if (searchTimer) clearTimeout(searchTimer);

		searchTimer = setTimeout(() => {
			query = { ...query, search: value.trim(), page: 1 };
			syncQueryUrl(query);
			void loadTransactions(query);
		}, 350);
	}

	function updateDateRangeFilter(from: string, to: string) {
		query = { ...query, from, to, page: 1 };
		syncQueryUrl(query);
		void loadTransactions(query);
	}

	function clearDateRangeFilter() {
		dateRangeValue = { start: undefined, end: undefined };
		isDateRangeOpen = false;
	}

	function updateTypeFilter(type: FinancialTransactionType | '') {
		query = { ...query, type, page: 1 };
		syncQueryUrl(query);
		void loadTransactions(query);
	}

	function updateStatusFilter(status: string[]) {
		const valid = financialTransactionStatuses.filter((item) => status.includes(item));
		query = { ...query, status: valid, page: 1 };
		syncQueryUrl(query);
		void loadTransactions(query);
	}

	function updateCategoryFilter(categoryId: string) {
		query = {
			...query,
			financial_category_id: categoryId ? Number(categoryId) : null,
			page: 1
		};
		syncQueryUrl(query);
		void loadTransactions(query);
	}

	function updateTransactionInList(updated: FinancialTransaction) {
		transactions = transactions.map((item) => (item.id === updated.id ? updated : item));
	}

	async function refreshAfterCreate() {
		query = { ...query, page: 1 };
		syncQueryUrl(query);
		await loadTransactions(query);
	}

	async function refreshAfterCategoriesChanged() {
		await loadCategories();
		await loadTransactions(query);
	}

	function getTypeFilterLabel() {
		if (query.type === 'income') return t('finances.transactions.typeIncome');
		if (query.type === 'expense') return t('finances.transactions.typeExpense');
		return t('finances.transactions.filterType');
	}

	function parseQuery(searchParams: URLSearchParams): TransactionQuery {
		const page = Number(searchParams.get('page') ?? defaultPagination.page);
		const limit = Number(searchParams.get('limit') ?? defaultPagination.pageSize);
		const search = searchParams.get('search') ?? '';
		const from = searchParams.get('from') ?? '';
		const to = searchParams.get('to') ?? '';
		const typeParam = searchParams.get('type') ?? '';
		const type: FinancialTransactionType | '' =
			typeParam === 'income' || typeParam === 'expense' ? typeParam : '';
		const categoryParam = Number(searchParams.get('category') ?? '');
		const status = (searchParams.get('status') ?? '')
			.split(',')
			.map((item) => item.trim())
			.filter(
				(item): item is FinancialTransactionStatus => item === 'completed' || item === 'cancelled'
			);

		return {
			page: Number.isFinite(page) ? Math.max(1, Math.trunc(page)) : defaultPagination.page,
			limit: pageSizeOptions.includes(limit) ? limit : defaultPagination.pageSize,
			search,
			from,
			to,
			type,
			financial_category_id:
				Number.isFinite(categoryParam) && categoryParam > 0 ? categoryParam : null,
			status
		};
	}

	function syncQueryUrl(nextQuery: TransactionQuery) {
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
		setOptionalSearchParam(nextUrl, 'type', nextQuery.type);
		setOptionalSearchParam(
			nextUrl,
			'category',
			nextQuery.financial_category_id ? String(nextQuery.financial_category_id) : ''
		);
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
		if (query.from) return `${t('common.from')} ${formatDate(query.from)}`;
		if (query.to) return `${t('common.until')} ${formatDate(query.to)}`;
		return t('common.selectDateRange');
	}

	function getErrorMessage(error: unknown) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return t('finances.transactions.errorLoad');
	}
</script>

<svelte:head>
	<title>{t('finances.transactions.title')} | PointFlow</title>
</svelte:head>

<div class="flex flex-col gap-6">
	<Card.Root>
		<Card.Header>
			<div>
				<Card.Title>{t('finances.transactions.title')}</Card.Title>
				<Card.Description>{t('finances.transactions.subtitle')}</Card.Description>
			</div>
			<Card.Action>
				<div class="flex w-full flex-col gap-2 sm:w-auto sm:flex-row">
					<Button
						variant="outline"
						class="w-full sm:w-auto"
						onclick={() => (isCategoriesOpen = true)}
					>
						<TagsIcon />
						{t('finances.transactions.manageCategoriesBtn')}
					</Button>
					<Button class="w-full sm:w-auto" onclick={() => (isCreateOpen = true)}>
						<PlusIcon />
						{t('finances.transactions.addBtn')}
					</Button>
				</div>
			</Card.Action>
		</Card.Header>

		<Card.Content class="flex flex-col gap-3">
			<SearchField
				value={searchValue}
				disabled={isLoading}
				ariaLabel={t('finances.transactions.searchAriaLabel')}
				placeholder={t('finances.transactions.searchPlaceholder')}
				onValueChange={updateSearchValue}
			/>

			<div class="flex flex-wrap items-center gap-2">
				<Popover.Root bind:open={isDateRangeOpen}>
					<Popover.Trigger
						class={cn(
							buttonVariants({ variant: 'outline' }),
							'h-9 justify-start gap-2 px-3 text-left font-normal sm:w-70',
							!query.from && !query.to && 'text-muted-foreground'
						)}
						disabled={isLoading}
						aria-label={t('common.filterByDateRange')}
					>
						<CalendarIcon class="size-4" />
						<span class="truncate">{getDateRangeLabel()}</span>
					</Popover.Trigger>
					<Popover.Content
						align="start"
						class="w-auto max-w-[calc(100vw-2rem)] overflow-x-auto p-0"
					>
						<RangeCalendar
							bind:value={dateRangeValue}
							class="rounded-md border-0 shadow-none"
							numberOfMonths={2}
						/>
						{#if query.from || query.to}
							<div class="flex justify-end border-t bg-popover p-2">
								<Button type="button" variant="ghost" size="sm" onclick={clearDateRangeFilter}>
									<XIcon />
									{t('common.clear')}
								</Button>
							</div>
						{/if}
					</Popover.Content>
				</Popover.Root>

				<Select.Root
					allowDeselect={false}
					type="single"
					value={query.type}
					disabled={isLoading}
					onValueChange={(value) => {
						if (value === 'income' || value === 'expense' || value === '') {
							updateTypeFilter(value);
						}
					}}
				>
					<Select.Trigger class="h-9 w-45 border-dashed" aria-label={getTypeFilterLabel()}>
						{#if query.type === 'income'}
							<BanknoteArrowDownIcon class="size-4" />
						{:else if query.type === 'expense'}
							<BanknoteArrowUpIcon class="size-4" />
						{/if}
						<span class="truncate">{getTypeFilterLabel()}</span>
					</Select.Trigger>
					<Select.Content>
						<Select.Item value="" label={t('finances.transactions.filterType')}>
							{t('finances.transactions.filterType')}
						</Select.Item>
						<Select.Item value="income" label={t('finances.transactions.typeIncome')}>
							{t('finances.transactions.typeIncome')}
						</Select.Item>
						<Select.Item value="expense" label={t('finances.transactions.typeExpense')}>
							{t('finances.transactions.typeExpense')}
						</Select.Item>
					</Select.Content>
				</Select.Root>

				<Select.Root
					allowDeselect={false}
					type="single"
					value={query.financial_category_id ? String(query.financial_category_id) : ''}
					disabled={isLoading}
					onValueChange={(value) => {
						if (typeof value === 'string') updateCategoryFilter(value);
					}}
				>
					<Select.Trigger
						class="h-9 w-52 border-dashed"
						aria-label={t('finances.transactions.filterCategory')}
					>
						<span class="truncate">
							{query.financial_category_id
								? (categories.find((item) => item.id === query.financial_category_id)?.name ??
									t('finances.transactions.filterCategory'))
								: t('finances.transactions.filterCategory')}
						</span>
					</Select.Trigger>
					<Select.Content>
						<Select.Item value="" label={t('common.all')}>
							{t('common.all')}
						</Select.Item>
						{#each categories as category (category.id)}
							<Select.Item value={String(category.id)} label={category.name}>
								{category.name}
							</Select.Item>
						{/each}
					</Select.Content>
				</Select.Root>

				<DataTableFacetedFilter
					title={t('finances.transactions.filterStatus')}
					options={statusFilterOptions}
					value={query.status}
					disabled={isLoading}
					onValueChange={updateStatusFilter}
				/>
			</div>

			{#if errorMessage && !isLoading}
				<ErrorAlert title={t('finances.transactions.errorLoad')} message={errorMessage} />
			{:else}
				<FinancialTransactionsTable
					{transactions}
					{isLoading}
					onTransactionUpdated={updateTransactionInList}
					onTransactionCancelled={updateTransactionInList}
					emptyMessage={query.search || query.from || query.to || query.type || query.status.length
						? t('finances.transactions.emptySearch')
						: t('finances.transactions.emptyDefault')}
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

	<FinancialTransactionCreateDialog bind:open={isCreateOpen} onCreated={refreshAfterCreate} />
	<FinancialCategoriesDialog
		bind:open={isCategoriesOpen}
		onChanged={refreshAfterCategoriesChanged}
	/>
</div>
