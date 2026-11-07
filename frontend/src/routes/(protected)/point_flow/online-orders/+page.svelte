<script lang="ts">
	import { replaceState } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import { onDestroy, onMount } from 'svelte';
	import RefreshCwIcon from '@lucide/svelte/icons/refresh-cw';
	import {
		listOnlineOrders,
		onlineOrderStatuses,
		type OnlineOrder,
		type OnlineOrderListParams,
		type OnlineOrderPagination,
		type OnlineOrderStatus
	} from '$lib/api/online-orders.js';
	import { ApiError } from '$lib/api/types.js';
	import DataPagination from '$lib/components/data-pagination.svelte';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import SearchField from '$lib/components/search-field.svelte';
	import { Button } from '$lib/components/ui/button/index.js';
	import * as Card from '$lib/components/ui/card/index.js';
	import { cn } from '$lib/utils.js';
	import OnlineOrdersTable from './components/online-orders-table.svelte';
	import { t } from '$lib/i18n/locale.svelte.js';

	const pageSizeOptions = [10, 25, 50, 100];
	const defaultPagination: OnlineOrderPagination = {
		page: 1,
		pageSize: 10,
		totalRecords: 0,
		totalPages: 0
	};
	const statusTabs: { value: OnlineOrderStatus | null; labelKey: string }[] = [
		{ value: null, labelKey: 'point_flow.onlineOrders.tabAll' },
		{ value: 'new', labelKey: 'point_flow.onlineOrders.statusNew' },
		{ value: 'packing', labelKey: 'point_flow.onlineOrders.statusPacking' },
		{ value: 'ready', labelKey: 'point_flow.onlineOrders.statusReady' },
		{ value: 'delivering', labelKey: 'point_flow.onlineOrders.statusDelivering' },
		{ value: 'completed', labelKey: 'point_flow.onlineOrders.statusCompleted' },
		{ value: 'cancelled', labelKey: 'point_flow.onlineOrders.statusCancelled' }
	];

	let orders = $state<OnlineOrder[]>([]);
	let pagination = $state<OnlineOrderPagination>(defaultPagination);
	let isLoading = $state(true);
	let errorMessage = $state<string | null>(null);
	const initialQuery = parseQuery(page.url.searchParams);
	let query = $state<Required<OnlineOrderListParams>>(initialQuery);
	let searchValue = $state(initialQuery.search);
	let latestRequestId = 0;
	let searchTimer: ReturnType<typeof setTimeout> | undefined;
	let refreshTimer: ReturnType<typeof setInterval> | undefined;

	onMount(() => {
		void loadOrders(query);
		refreshTimer = setInterval(() => {
			if (document.hidden) return;
			void loadOrders(query, { silent: true });
		}, 30000);
	});

	onDestroy(() => {
		if (searchTimer) clearTimeout(searchTimer);
		if (refreshTimer) clearInterval(refreshTimer);
	});

	async function loadOrders(params: OnlineOrderListParams, options: { silent?: boolean } = {}) {
		const requestId = ++latestRequestId;
		if (!options.silent) {
			isLoading = true;
		}
		if (!options.silent) {
			errorMessage = null;
		}

		try {
			const result = await listOnlineOrders(params);
			if (requestId !== latestRequestId) return;

			if (
				result.pagination.totalPages > 0 &&
				result.pagination.page > result.pagination.totalPages
			) {
				query = { ...query, page: result.pagination.totalPages };
				syncQueryUrl(query);
				await loadOrders(query);
				return;
			}

			orders = result.orders;
			pagination = result.pagination;
			if (options.silent) {
				errorMessage = null;
			}
		} catch (error) {
			if (requestId !== latestRequestId) return;
			if (!options.silent) {
				errorMessage = getErrorMessage(error);
				orders = [];
			}
		} finally {
			if (requestId === latestRequestId && !options.silent) {
				isLoading = false;
			}
		}
	}

	function updatePageSize(pageSize: number) {
		if (!pageSizeOptions.includes(pageSize)) return;

		query = { ...query, limit: pageSize, page: 1 };
		syncQueryUrl(query);
		void loadOrders(query);
	}

	function updatePage(nextPage: number) {
		const next = Math.max(1, Math.trunc(nextPage));
		if (next === query.page) return;

		query = { ...query, page: next };
		syncQueryUrl(query);
		void loadOrders(query);
	}

	function updateSearchValue(value: string) {
		searchValue = value;
		if (searchTimer) clearTimeout(searchTimer);

		searchTimer = setTimeout(() => {
			query = { ...query, search: value.trim(), page: 1 };
			syncQueryUrl(query);
			void loadOrders(query);
		}, 350);
	}

	function selectStatus(status: OnlineOrderStatus | null) {
		query = { ...query, status: status ? [status] : [], page: 1 };
		syncQueryUrl(query);
		void loadOrders(query);
	}

	function refreshNow() {
		void loadOrders(query);
	}

	async function refreshAfterChange() {
		await loadOrders(query);
	}

	function parseQuery(searchParams: URLSearchParams): Required<OnlineOrderListParams> {
		const rawPage = Number(searchParams.get('page') ?? defaultPagination.page);
		const rawLimit = Number(searchParams.get('limit') ?? defaultPagination.pageSize);
		const search = searchParams.get('search') ?? '';
		const status = parseStatusQuery(searchParams.get('status') ?? '');

		return {
			page: Number.isFinite(rawPage) ? Math.max(1, Math.trunc(rawPage)) : defaultPagination.page,
			limit: pageSizeOptions.includes(rawLimit) ? rawLimit : defaultPagination.pageSize,
			search,
			status
		};
	}

	function syncQueryUrl(nextQuery: Required<OnlineOrderListParams>) {
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

	function parseStatusQuery(value: string): OnlineOrderStatus[] {
		const first = value.split(',').map((part) => part.trim())[0] ?? '';
		if (isOrderStatus(first)) return [first];
		return [];
	}

	function isOrderStatus(value: string): value is OnlineOrderStatus {
		return (onlineOrderStatuses as readonly string[]).includes(value);
	}

	function getErrorMessage(error: unknown) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return t('point_flow.onlineOrders.errorLoad');
	}

	const activeTab = $derived(query.status[0] ?? null);
</script>

<svelte:head>
	<title>{t('point_flow.onlineOrders.title')} | PointFlow</title>
</svelte:head>

<div class="flex flex-col gap-6">
	<Card.Root>
		<Card.Header>
			<div>
				<Card.Title>{t('point_flow.onlineOrders.title')}</Card.Title>
				<Card.Description>{t('point_flow.onlineOrders.subtitle')}</Card.Description>
			</div>
			<Card.Action>
				<Button
					variant="outline"
					class="w-full sm:w-auto"
					onclick={refreshNow}
					disabled={isLoading}
				>
					<RefreshCwIcon class={cn(isLoading && 'animate-spin')} />
					{t('point_flow.onlineOrders.refresh')}
				</Button>
			</Card.Action>
		</Card.Header>

		<Card.Content class="flex flex-col gap-3">
			<div class="flex flex-col gap-2">
				<SearchField
					value={searchValue}
					disabled={isLoading}
					ariaLabel={t('point_flow.onlineOrders.searchAriaLabel')}
					placeholder={t('point_flow.onlineOrders.searchPlaceholder')}
					class="sm:max-w-sm"
					onValueChange={updateSearchValue}
				/>
				<div
					class="flex flex-wrap gap-2"
					role="tablist"
					aria-label={t('point_flow.onlineOrders.filterStatus')}
				>
					{#each statusTabs as tab (tab.labelKey)}
						<Button
							type="button"
							variant={activeTab === tab.value ? 'default' : 'outline'}
							size="sm"
							role="tab"
							aria-selected={activeTab === tab.value}
							disabled={isLoading}
							onclick={() => selectStatus(tab.value)}
						>
							{t(tab.labelKey)}
						</Button>
					{/each}
				</div>
				<p class="text-xs text-muted-foreground">{t('point_flow.onlineOrders.autoRefreshNote')}</p>
			</div>

			{#if errorMessage && !isLoading}
				<ErrorAlert title={t('point_flow.onlineOrders.errorLoad')} message={errorMessage} />
			{:else}
				<OnlineOrdersTable
					{orders}
					{isLoading}
					onOrderChanged={refreshAfterChange}
					emptyMessage={query.search || query.status.length
						? t('point_flow.onlineOrders.emptySearch')
						: t('point_flow.onlineOrders.emptyDefault')}
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
