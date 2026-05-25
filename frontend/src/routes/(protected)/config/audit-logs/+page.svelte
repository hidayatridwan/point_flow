<script lang="ts">
	import { replaceState } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import { onDestroy, onMount } from 'svelte';
	import CalendarIcon from '@lucide/svelte/icons/calendar';
	import CircleCheckIcon from '@lucide/svelte/icons/circle-check';
	import CircleOffIcon from '@lucide/svelte/icons/circle-off';
	import RefreshCwIcon from '@lucide/svelte/icons/refresh-cw';
	import XIcon from '@lucide/svelte/icons/x';
	import { parseDate, type DateValue } from '@internationalized/date';
	import type { DateRange } from 'bits-ui';
	import {
		auditLogActions,
		auditLogOutcomes,
		auditLogResourceTypes,
		listAuditLogs,
		type AuditLog,
		type AuditLogAction,
		type AuditLogListParams,
		type AuditLogOutcome,
		type AuditLogPagination,
		type AuditLogResourceType
	} from '$lib/api/audit-logs.js';
	import { ApiError } from '$lib/api/types.js';
	import DataTableFacetedFilter from '$lib/components/data-table-faceted-filter.svelte';
	import DataPagination from '$lib/components/data-pagination.svelte';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import PageHeader from '$lib/components/page-header.svelte';
	import SearchField from '$lib/components/search-field.svelte';
	import { Button, buttonVariants } from '$lib/components/ui/button/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import * as Popover from '$lib/components/ui/popover/index.js';
	import RangeCalendar from '$lib/components/ui/range-calendar/range-calendar.svelte';
	import { formatDate } from '$lib/formatters.js';
	import { cn } from '$lib/utils.js';
	import AuditLogsTable from './components/audit-logs-table.svelte';

	type AuditQuery = Required<Omit<AuditLogListParams, 'actor_id'>> & {
		actor_id: number | null;
	};

	const pageSizeOptions = [10, 25, 50, 100];
	const defaultPagination: AuditLogPagination = {
		page: 1,
		pageSize: 10,
		totalRecords: 0,
		totalPages: 0
	};
	const outcomeFilterOptions = [
		{ value: 'success', label: 'Success', icon: CircleCheckIcon },
		{ value: 'failure', label: 'Failure', icon: CircleOffIcon }
	];
	const actionFilterOptions = [
		{ value: 'auth.login', label: 'Login' },
		{ value: 'auth.logout', label: 'Logout' },
		{ value: 'auth.bootstrap_admin', label: 'Bootstrap Admin' },
		{ value: 'auth.change_password', label: 'Change Password' },
		{ value: 'cash_register_session.open', label: 'Open Register' },
		{ value: 'cash_register_session.close', label: 'Close Register' },
		{ value: 'cash_drawer_movement.create', label: 'Create Cash Movement' },
		{ value: 'cash_drawer_movement.cancel', label: 'Cancel Cash Movement' },
		{ value: 'sale.create', label: 'Create Sale' },
		{ value: 'sale.cancel', label: 'Cancel Sale' },
		{ value: 'sale_return.create', label: 'Create Return' },
		{ value: 'sale_return.cancel', label: 'Cancel Return' }
	];
	const resourceFilterOptions = [
		{ value: 'user', label: 'User' },
		{ value: 'cash_register_session', label: 'Register Session' },
		{ value: 'cash_drawer_movement', label: 'Cash Movement' },
		{ value: 'sale', label: 'Sale' },
		{ value: 'sale_return', label: 'Sale Return' }
	];
	const outcomeFilterValues = new Set(outcomeFilterOptions.map((option) => option.value));
	const actionFilterValues = new Set(actionFilterOptions.map((option) => option.value));
	const resourceFilterValues = new Set(resourceFilterOptions.map((option) => option.value));

	let auditLogs = $state<AuditLog[]>([]);
	let pagination = $state<AuditLogPagination>(defaultPagination);
	let isLoading = $state(true);
	let errorMessage = $state<string | null>(null);
	const initialQuery = parseQuery(page.url.searchParams);
	let query = $state<AuditQuery>(initialQuery);
	let searchValue = $state(initialQuery.search);
	let requestIdValue = $state(initialQuery.request_id);
	let actorIdValue = $state(initialQuery.actor_id ? String(initialQuery.actor_id) : '');
	let dateRangeValue = $state<DateRange>({
		start: dateStringToDateValue(initialQuery.from),
		end: dateStringToDateValue(initialQuery.to)
	});
	let isDateRangeOpen = $state(false);
	let latestRequestId = 0;
	let filterTimer: ReturnType<typeof setTimeout> | undefined;

	onMount(() => {
		void loadAuditLogs(query);
	});

	onDestroy(() => {
		if (filterTimer) clearTimeout(filterTimer);
	});

	$effect(() => {
		const from = dateValueToDateString(dateRangeValue.start);
		const to = dateValueToDateString(dateRangeValue.end);
		if (from === query.from && to === query.to) return;

		updateDateRangeFilter(from, to);
	});

	async function loadAuditLogs(params: AuditLogListParams) {
		const requestId = ++latestRequestId;
		isLoading = true;
		errorMessage = null;

		try {
			const result = await listAuditLogs(params);
			if (requestId !== latestRequestId) return;

			if (
				result.pagination.totalPages > 0 &&
				result.pagination.page > result.pagination.totalPages
			) {
				query = { ...query, page: result.pagination.totalPages };
				syncQueryUrl(query);
				await loadAuditLogs(query);
				return;
			}

			auditLogs = result.auditLogs;
			pagination = result.pagination;
		} catch (error) {
			if (requestId !== latestRequestId) return;
			errorMessage = getErrorMessage(error);
			auditLogs = [];
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
		void loadAuditLogs(query);
	}

	function updatePage(nextPage: number) {
		const page = Math.max(1, Math.trunc(nextPage));
		if (page === query.page) return;

		query = { ...query, page };
		syncQueryUrl(query);
		void loadAuditLogs(query);
	}

	function updateSearchValue(value: string) {
		searchValue = value;
		debounceFilterUpdate(() => {
			query = { ...query, search: value.trim(), page: 1 };
			syncQueryUrl(query);
			void loadAuditLogs(query);
		});
	}

	function updateRequestIdValue(value: string) {
		requestIdValue = value;
		debounceFilterUpdate(() => {
			query = { ...query, request_id: value.trim(), page: 1 };
			syncQueryUrl(query);
			void loadAuditLogs(query);
		});
	}

	function updateActorIdValue(value: string) {
		actorIdValue = value;
		debounceFilterUpdate(() => {
			query = { ...query, actor_id: parseActorId(value), page: 1 };
			syncQueryUrl(query);
			void loadAuditLogs(query);
		});
	}

	function debounceFilterUpdate(callback: () => void) {
		if (filterTimer) clearTimeout(filterTimer);
		filterTimer = setTimeout(callback, 350);
	}

	function updateDateRangeFilter(from: string, to: string) {
		query = { ...query, from, to, page: 1 };
		syncQueryUrl(query);
		void loadAuditLogs(query);
	}

	function updateOutcomeFilter(value: string[]) {
		query = { ...query, outcome: value.filter(isOutcomeFilterValue), page: 1 };
		syncQueryUrl(query);
		void loadAuditLogs(query);
	}

	function updateActionFilter(value: string[]) {
		query = { ...query, action: value.filter(isActionFilterValue), page: 1 };
		syncQueryUrl(query);
		void loadAuditLogs(query);
	}

	function updateResourceFilter(value: string[]) {
		query = { ...query, resource_type: value.filter(isResourceFilterValue), page: 1 };
		syncQueryUrl(query);
		void loadAuditLogs(query);
	}

	function clearDateRangeFilter() {
		dateRangeValue = { start: undefined, end: undefined };
		isDateRangeOpen = false;
	}

	function refreshAuditLogs() {
		void loadAuditLogs(query);
	}

	function parseQuery(searchParams: URLSearchParams): AuditQuery {
		const page = Number(searchParams.get('page') ?? defaultPagination.page);
		const limit = Number(searchParams.get('limit') ?? defaultPagination.pageSize);
		const search = searchParams.get('search') ?? '';
		const from = searchParams.get('from') ?? '';
		const to = searchParams.get('to') ?? '';
		const action = parseActionQuery(searchParams.get('action') ?? '');
		const resourceType = parseResourceQuery(searchParams.get('resource_type') ?? '');
		const outcome = parseOutcomeQuery(searchParams.get('outcome') ?? '');
		const requestId = searchParams.get('request_id') ?? '';
		const actorId = parseActorId(searchParams.get('actor_id') ?? '');

		return {
			page: Number.isFinite(page) ? Math.max(1, Math.trunc(page)) : defaultPagination.page,
			limit: pageSizeOptions.includes(limit) ? limit : defaultPagination.pageSize,
			search,
			from,
			to,
			action,
			resource_type: resourceType,
			outcome,
			request_id: requestId,
			actor_id: actorId
		};
	}

	function syncQueryUrl(nextQuery: AuditQuery) {
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
		setOptionalSearchParam(nextUrl, 'action', nextQuery.action.join(','));
		setOptionalSearchParam(nextUrl, 'resource_type', nextQuery.resource_type.join(','));
		setOptionalSearchParam(nextUrl, 'outcome', nextQuery.outcome.join(','));
		setOptionalSearchParam(nextUrl, 'request_id', nextQuery.request_id);

		if (nextQuery.actor_id) {
			nextUrl.searchParams.set('actor_id', String(nextQuery.actor_id));
		} else {
			nextUrl.searchParams.delete('actor_id');
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

	function parseActorId(value: string) {
		if (!value.trim()) return null;
		const actorId = Number(value);
		return Number.isFinite(actorId) && actorId > 0 ? Math.trunc(actorId) : null;
	}

	function parseActionQuery(value: string): AuditLogAction[] {
		return value.split(',').filter(isActionFilterValue);
	}

	function parseResourceQuery(value: string): AuditLogResourceType[] {
		return value.split(',').filter(isResourceFilterValue);
	}

	function parseOutcomeQuery(value: string): AuditLogOutcome[] {
		return value.split(',').filter(isOutcomeFilterValue);
	}

	function isActionFilterValue(value: string): value is AuditLogAction {
		return actionFilterValues.has(value) && auditLogActions.includes(value as AuditLogAction);
	}

	function isResourceFilterValue(value: string): value is AuditLogResourceType {
		return (
			resourceFilterValues.has(value) &&
			auditLogResourceTypes.includes(value as AuditLogResourceType)
		);
	}

	function isOutcomeFilterValue(value: string): value is AuditLogOutcome {
		return outcomeFilterValues.has(value) && auditLogOutcomes.includes(value as AuditLogOutcome);
	}

	function getErrorMessage(error: unknown) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return 'Failed to load audit logs.';
	}
</script>

<svelte:head>
	<title>Audit Logs | POS System</title>
</svelte:head>

<div class="flex flex-col gap-6">
	<PageHeader title="Audit Logs" description="Review append-only security and operational events.">
		{#snippet actions()}
			<Button
				type="button"
				variant="outline"
				class="w-full sm:w-auto"
				disabled={isLoading}
				onclick={refreshAuditLogs}
			>
				<RefreshCwIcon />
				Refresh
			</Button>
		{/snippet}
	</PageHeader>

	<section class="flex flex-col gap-3" aria-labelledby="audit-logs-table-title">
		<h2 id="audit-logs-table-title" class="sr-only">Audit log records</h2>

		<div class="grid gap-3 xl:grid-cols-[minmax(0,1fr)_auto] xl:items-start">
			<div class="flex flex-col gap-2 sm:flex-row sm:flex-wrap sm:items-center">
				<SearchField
					value={searchValue}
					disabled={isLoading}
					ariaLabel="Search audit logs"
					placeholder="Search audit logs..."
					class="sm:max-w-sm"
					onValueChange={updateSearchValue}
				/>
				<DataTableFacetedFilter
					title="Outcome"
					options={outcomeFilterOptions}
					value={query.outcome}
					disabled={isLoading}
					onValueChange={updateOutcomeFilter}
				/>
				<DataTableFacetedFilter
					title="Action"
					options={actionFilterOptions}
					value={query.action}
					disabled={isLoading}
					onValueChange={updateActionFilter}
				/>
				<DataTableFacetedFilter
					title="Resource"
					options={resourceFilterOptions}
					value={query.resource_type}
					disabled={isLoading}
					onValueChange={updateResourceFilter}
				/>
				<Input
					value={requestIdValue}
					placeholder="Request ID"
					aria-label="Filter audit logs by request ID"
					class="h-9 sm:w-64"
					disabled={isLoading}
					oninput={(event) => updateRequestIdValue(event.currentTarget.value)}
				/>
				<Input
					value={actorIdValue}
					type="number"
					min="1"
					step="1"
					placeholder="Actor ID"
					aria-label="Filter audit logs by actor ID"
					class="h-9 sm:w-32"
					disabled={isLoading}
					oninput={(event) => updateActorIdValue(event.currentTarget.value)}
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
					aria-label="Filter audit logs by date range"
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
			<ErrorAlert title="Failed to load audit logs." message={errorMessage} />
		{:else}
			<AuditLogsTable
				{auditLogs}
				{isLoading}
				emptyMessage={query.search ||
				query.outcome.length ||
				query.action.length ||
				query.resource_type.length ||
				query.request_id ||
				query.actor_id ||
				query.from ||
				query.to
					? 'No audit logs match your filters.'
					: 'No audit logs yet.'}
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
