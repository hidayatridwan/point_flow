<script lang="ts">
	import { replaceState } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import { onDestroy, onMount } from 'svelte';
	import BanknoteIcon from '@lucide/svelte/icons/banknote';
	import BanknoteArrowDownIcon from '@lucide/svelte/icons/banknote-arrow-down';
	import BanknoteArrowUpIcon from '@lucide/svelte/icons/banknote-arrow-up';
	import CalendarIcon from '@lucide/svelte/icons/calendar';
	import CircleCheckIcon from '@lucide/svelte/icons/circle-check';
	import CircleOffIcon from '@lucide/svelte/icons/circle-off';
	import HandCoinsIcon from '@lucide/svelte/icons/hand-coins';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import RefreshCwIcon from '@lucide/svelte/icons/refresh-cw';
	import WalletCardsIcon from '@lucide/svelte/icons/wallet-cards';
	import XIcon from '@lucide/svelte/icons/x';
	import { parseDate, type DateValue } from '@internationalized/date';
	import type { DateRange } from 'bits-ui';
	import {
		cashDrawerMovementStatuses,
		cashDrawerMovementTypes,
		clearStoredActiveCashRegisterSessionId,
		getStoredActiveCashRegisterSessionId,
		listCashDrawerMovements,
		listCashRegisterSessions,
		setStoredActiveCashRegisterSessionId,
		type CashDrawerMovement,
		type CashDrawerMovementListParams,
		type CashDrawerMovementStatus,
		type CashDrawerMovementType,
		type CashRegisterPagination,
		type CashRegisterSession
	} from '$lib/api/cash-registers.js';
	import { ApiError } from '$lib/api/types.js';
	import DataTableFacetedFilter from '$lib/components/data-table-faceted-filter.svelte';
	import DataPagination from '$lib/components/data-pagination.svelte';
	import DetailField from '$lib/components/detail-field.svelte';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import PageHeader from '$lib/components/page-header.svelte';
	import SearchField from '$lib/components/search-field.svelte';
	import { Button, buttonVariants } from '$lib/components/ui/button/index.js';
	import * as Popover from '$lib/components/ui/popover/index.js';
	import RangeCalendar from '$lib/components/ui/range-calendar/range-calendar.svelte';
	import * as Select from '$lib/components/ui/select/index.js';
	import { Spinner } from '$lib/components/ui/spinner/index.js';
	import { formatDate, formatDateTime, formatIdr } from '$lib/formatters.js';
	import { cn } from '$lib/utils.js';
	import CashDrawerMovementCreateDialog from './components/cash-drawer-movement-create-dialog.svelte';
	import CashDrawerMovementsTable from './components/cash-drawer-movements-table.svelte';
	import { t } from '$lib/i18n/locale.svelte.js';

	type MovementQuery = Required<Omit<CashDrawerMovementListParams, 'cash_register_session_id'>> & {
		cash_register_session_id: number | null;
	};

	const pageSizeOptions = [10, 25, 50, 100];
	const defaultPagination: CashRegisterPagination = {
		page: 1,
		pageSize: 10,
		totalRecords: 0,
		totalPages: 0
	};
	const statusFilterOptions = [
		{ value: 'completed', label: t('pos.cashMovements.statusCompleted'), icon: CircleCheckIcon },
		{ value: 'cancelled', label: t('pos.cashMovements.statusCancelled'), icon: CircleOffIcon }
	];
	const typeFilterOptions = [
		{ value: 'cash_in', label: t('pos.cashMovements.typeCashIn'), icon: BanknoteArrowDownIcon },
		{ value: 'cash_drop', label: t('pos.cashMovements.typeCashDrop'), icon: BanknoteArrowUpIcon },
		{ value: 'cash_out', label: t('pos.cashMovements.typeCashOut'), icon: WalletCardsIcon },
		{ value: 'petty_cash', label: t('pos.cashMovements.typePettyCash'), icon: HandCoinsIcon }
	];
	const statusFilterValues = new Set(statusFilterOptions.map((option) => option.value));
	const typeFilterValues = new Set(typeFilterOptions.map((option) => option.value));

	let openSessions = $state<CashRegisterSession[]>([]);
	let activeSession = $state<CashRegisterSession | null>(null);
	let movements = $state<CashDrawerMovement[]>([]);
	let pagination = $state<CashRegisterPagination>(defaultPagination);
	let isSessionLoading = $state(true);
	let isLoading = $state(true);
	let errorMessage = $state<string | null>(null);
	let sessionErrorMessage = $state<string | null>(null);
	let isCreateOpen = $state(false);
	const initialQuery = parseQuery(page.url.searchParams);
	let query = $state<MovementQuery>(initialQuery);
	let searchValue = $state(initialQuery.search);
	let dateRangeValue = $state<DateRange>({
		start: dateStringToDateValue(initialQuery.from),
		end: dateStringToDateValue(initialQuery.to)
	});
	let isDateRangeOpen = $state(false);
	let latestRequestId = 0;
	let latestSessionRequestId = 0;
	let searchTimer: ReturnType<typeof setTimeout> | undefined;

	const isCurrentSessionFilterActive = $derived(
		Boolean(activeSession && query.cash_register_session_id === activeSession.id)
	);

	onMount(() => {
		void loadOpenSessions();
		void loadMovements(query);
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

	async function loadOpenSessions() {
		const requestId = ++latestSessionRequestId;
		isSessionLoading = true;
		sessionErrorMessage = null;

		try {
			const result = await listCashRegisterSessions({ limit: 100, status: ['open'] });
			if (requestId !== latestSessionRequestId) return;

			openSessions = result.sessions;
			activeSession = resolveActiveSession(result.sessions);

			if (activeSession) {
				setStoredActiveCashRegisterSessionId(activeSession.id);
			} else {
				clearStoredActiveCashRegisterSessionId();
			}
		} catch (error) {
			if (requestId !== latestSessionRequestId) return;
			openSessions = [];
			activeSession = null;
			sessionErrorMessage = getErrorMessage(error, 'Failed to load open register sessions.');
		} finally {
			if (requestId === latestSessionRequestId) isSessionLoading = false;
		}
	}

	async function loadMovements(params: CashDrawerMovementListParams) {
		const requestId = ++latestRequestId;
		isLoading = true;
		errorMessage = null;

		try {
			const result = await listCashDrawerMovements(params);
			if (requestId !== latestRequestId) return;

			if (
				result.pagination.totalPages > 0 &&
				result.pagination.page > result.pagination.totalPages
			) {
				query = { ...query, page: result.pagination.totalPages };
				syncQueryUrl(query);
				await loadMovements(query);
				return;
			}

			movements = result.movements;
			pagination = result.pagination;
		} catch (error) {
			if (requestId !== latestRequestId) return;
			errorMessage = getErrorMessage(error, 'Failed to load cash drawer movements.');
			movements = [];
		} finally {
			if (requestId === latestRequestId) {
				isLoading = false;
			}
		}
	}

	async function refreshPageState() {
		await Promise.all([loadOpenSessions(), loadMovements(query)]);
	}

	function chooseOpenSession(sessionId: string) {
		const session = openSessions.find((session) => String(session.id) === sessionId);
		if (!session) return;
		activeSession = session;
		setStoredActiveCashRegisterSessionId(session.id);
	}

	function toggleCurrentSessionFilter() {
		const sessionId = activeSession?.id ?? null;
		query = {
			...query,
			cash_register_session_id: isCurrentSessionFilterActive ? null : sessionId,
			page: 1
		};
		syncQueryUrl(query);
		void loadMovements(query);
	}

	function openCreateDialog() {
		isCreateOpen = true;
	}

	async function refreshMovementsAfterChange() {
		await loadMovements(query);
	}

	async function handleMovementCreated() {
		await loadMovements(query);
	}

	function updatePageSize(pageSize: number) {
		if (!pageSizeOptions.includes(pageSize)) return;

		query = { ...query, limit: pageSize, page: 1 };
		syncQueryUrl(query);
		void loadMovements(query);
	}

	function updatePage(nextPage: number) {
		const page = Math.max(1, Math.trunc(nextPage));
		if (page === query.page) return;

		query = { ...query, page };
		syncQueryUrl(query);
		void loadMovements(query);
	}

	function updateSearchValue(value: string) {
		searchValue = value;
		if (searchTimer) clearTimeout(searchTimer);

		searchTimer = setTimeout(() => {
			query = { ...query, search: value.trim(), page: 1 };
			syncQueryUrl(query);
			void loadMovements(query);
		}, 350);
	}

	function updateDateRangeFilter(from: string, to: string) {
		query = { ...query, from, to, page: 1 };
		syncQueryUrl(query);
		void loadMovements(query);
	}

	function updateStatusFilter(value: string[]) {
		query = { ...query, status: value.filter(isStatusFilterValue), page: 1 };
		syncQueryUrl(query);
		void loadMovements(query);
	}

	function updateTypeFilter(value: string[]) {
		query = { ...query, type: value.filter(isTypeFilterValue), page: 1 };
		syncQueryUrl(query);
		void loadMovements(query);
	}

	function clearDateRangeFilter() {
		dateRangeValue = { start: undefined, end: undefined };
		isDateRangeOpen = false;
	}

	function resolveActiveSession(sessions: CashRegisterSession[]) {
		const storedSessionId = getStoredActiveCashRegisterSessionId();
		if (storedSessionId) {
			const storedSession = sessions.find((session) => session.id === storedSessionId);
			if (storedSession) return storedSession;
		}
		return sessions[0] ?? null;
	}

	function parseQuery(searchParams: URLSearchParams): MovementQuery {
		const page = Number(searchParams.get('page') ?? defaultPagination.page);
		const limit = Number(searchParams.get('limit') ?? defaultPagination.pageSize);
		const search = searchParams.get('search') ?? '';
		const from = searchParams.get('from') ?? '';
		const to = searchParams.get('to') ?? '';
		const status = parseStatusQuery(searchParams.get('status') ?? '');
		const type = parseTypeQuery(searchParams.get('type') ?? '');
		const sessionId = Number(searchParams.get('session_id') ?? '');

		return {
			page: Number.isFinite(page) ? Math.max(1, Math.trunc(page)) : defaultPagination.page,
			limit: pageSizeOptions.includes(limit) ? limit : defaultPagination.pageSize,
			search,
			from,
			to,
			status,
			type,
			cash_register_session_id:
				Number.isFinite(sessionId) && sessionId > 0 ? Math.trunc(sessionId) : null
		};
	}

	function syncQueryUrl(nextQuery: MovementQuery) {
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
		setOptionalSearchParam(nextUrl, 'type', nextQuery.type.join(','));

		if (nextQuery.cash_register_session_id) {
			nextUrl.searchParams.set('session_id', String(nextQuery.cash_register_session_id));
		} else {
			nextUrl.searchParams.delete('session_id');
		}

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

	function parseStatusQuery(value: string): CashDrawerMovementStatus[] {
		return value.split(',').filter(isStatusFilterValue);
	}

	function parseTypeQuery(value: string): CashDrawerMovementType[] {
		return value.split(',').filter(isTypeFilterValue);
	}

	function isStatusFilterValue(value: string): value is CashDrawerMovementStatus {
		return (
			statusFilterValues.has(value) &&
			cashDrawerMovementStatuses.includes(value as CashDrawerMovementStatus)
		);
	}

	function isTypeFilterValue(value: string): value is CashDrawerMovementType {
		return typeFilterValues.has(value) && cashDrawerMovementTypes.includes(value as CashDrawerMovementType);
	}

	function getErrorMessage(error: unknown, fallback: string) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return fallback;
	}
</script>

<svelte:head>
	<title>Cash Drawer Movements | POS System</title>
</svelte:head>

<div class="flex flex-col gap-6">
	<PageHeader
		title={t('pos.cashMovements.title')}
		description={t('pos.cashMovements.subtitle')}
	>
		{#snippet actions()}
			<div class="flex w-full flex-col gap-2 sm:w-auto sm:flex-row">
				<Button
					type="button"
					variant="outline"
					class="w-full sm:w-auto"
					onclick={refreshPageState}
				>
					<RefreshCwIcon />
					{t('common.refresh')}
				</Button>
				<Button class="w-full sm:w-auto" href="/pos/register-session">
					<BanknoteIcon />
					{t('pos.cashMovements.registerSessionBtn')}
				</Button>
				<Button
					type="button"
					class="w-full sm:w-auto"
					disabled={!activeSession || isSessionLoading}
					onclick={openCreateDialog}
				>
					<PlusIcon />
					{t('pos.cashMovements.newMovementBtn')}
				</Button>
			</div>
		{/snippet}
	</PageHeader>

	<section class="grid gap-3 lg:grid-cols-[minmax(0,1fr)_360px]">
		<div class="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
			<DetailField label={t('pos.cashMovements.activeSession')} value={activeSession?.session_no ?? '-'} />
			<DetailField label="Register" value={activeSession?.cash_register_code ?? '-'} />
			<DetailField label={t('pos.registerSession.openedAt')} value={formatDateTime(activeSession?.opened_at)} />
			<DetailField label={t('pos.registerSession.openingCash')} value={formatIdr(activeSession?.opening_cash)} />
		</div>

		<div class="grid content-start gap-3 rounded-md border p-4">
			<div class="flex items-center gap-2">
				<BanknoteIcon class="size-5 text-muted-foreground" />
				<h2 class="text-base font-semibold">{t('pos.cashMovements.drawerSession')}</h2>
			</div>

			{#if isSessionLoading}
				<div class="flex h-9 items-center gap-2 text-sm text-muted-foreground">
					<Spinner />
					{t('pos.cashMovements.loadingSessions')}
				</div>
			{:else if sessionErrorMessage}
				<ErrorAlert title={t('pos.cashMovements.errorLoadSessions')} message={sessionErrorMessage} />
			{:else if openSessions.length > 0}
				<Select.Root
					allowDeselect={false}
					type="single"
					value={activeSession ? String(activeSession.id) : ''}
					onValueChange={chooseOpenSession}
				>
					<Select.Trigger class="w-full">
						<Select.Value placeholder={t('pos.cashMovements.selectSession')} />
					</Select.Trigger>
					<Select.Content>
						{#each openSessions as session (session.id)}
							<Select.Item
								value={String(session.id)}
								label={`${session.cash_register_code} - ${session.session_no}`}
							>
								{session.cash_register_code} - {session.session_no}
							</Select.Item>
						{/each}
					</Select.Content>
				</Select.Root>
				<p class="text-sm text-muted-foreground">
					{t('pos.cashMovements.newMovementsHint')}
				</p>
			{:else}
				<ErrorAlert
					title={t('pos.cashMovements.noActiveRegister')}
					message={t('pos.cashMovements.noActiveRegisterDesc')}
				/>
			{/if}
		</div>
	</section>

	<section class="flex flex-col gap-3" aria-labelledby="cash-drawer-movements-table-title">
		<h2 id="cash-drawer-movements-table-title" class="sr-only">{t('pos.cashMovements.srHeading')}</h2>

		<div class="grid gap-3 xl:grid-cols-[minmax(0,1fr)_auto] xl:items-center">
			<div class="flex flex-col gap-2 sm:flex-row sm:flex-wrap sm:items-center">
				<SearchField
					value={searchValue}
					disabled={isLoading}
					ariaLabel={t('pos.cashMovements.searchAriaLabel')}
					placeholder={t('pos.cashMovements.searchPlaceholder')}
					class="sm:max-w-sm"
					onValueChange={updateSearchValue}
				/>
				<DataTableFacetedFilter
					title={t('pos.cashMovements.filterStatus')}
					options={statusFilterOptions}
					value={query.status}
					disabled={isLoading}
					onValueChange={updateStatusFilter}
				/>
				<DataTableFacetedFilter
					title={t('pos.cashMovements.filterType')}
					options={typeFilterOptions}
					value={query.type}
					disabled={isLoading}
					onValueChange={updateTypeFilter}
				/>
				<Button
					type="button"
					variant={isCurrentSessionFilterActive ? 'secondary' : 'outline'}
					size="sm"
					class="h-9 justify-start"
					disabled={!activeSession || isLoading}
					onclick={toggleCurrentSessionFilter}
				>
					<BanknoteIcon />
					{t('pos.cashMovements.currentSession')}
				</Button>
			</div>

			<Popover.Root bind:open={isDateRangeOpen}>
				<Popover.Trigger
					class={cn(
						buttonVariants({ variant: 'outline' }),
						'h-9 w-full justify-start gap-2 px-3 text-left font-normal sm:w-[280px]',
						!query.from && !query.to && 'text-muted-foreground'
					)}
					disabled={isLoading}
					aria-label="Filter cash drawer movements by date range"
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
			<ErrorAlert title={t('pos.cashMovements.errorLoad')} message={errorMessage} />
		{:else}
			<CashDrawerMovementsTable
				{movements}
				{isLoading}
				onMovementChanged={refreshMovementsAfterChange}
				emptyMessage={query.search ||
				query.status.length ||
				query.type.length ||
				query.from ||
				query.to ||
				query.cash_register_session_id
					? t('pos.cashMovements.emptySearch')
					: t('pos.cashMovements.emptyDefault')}
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

	<CashDrawerMovementCreateDialog
		{activeSession}
		bind:open={isCreateOpen}
		onCreated={handleMovementCreated}
	/>
</div>
