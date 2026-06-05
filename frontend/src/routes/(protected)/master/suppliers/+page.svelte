<script lang="ts">
	import { replaceState } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import { onDestroy, onMount } from 'svelte';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import {
		listSuppliers,
		type Supplier,
		type SupplierListParams,
		type SupplierPagination
	} from '$lib/api/suppliers.js';
	import { ApiError } from '$lib/api/types.js';
	import DataPagination from '$lib/components/data-pagination.svelte';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import * as Card from '$lib/components/ui/card/index.js';
	import SearchField from '$lib/components/search-field.svelte';
	import { Button } from '$lib/components/ui/button/index.js';
	import SupplierCreateDialog from './components/supplier-create-dialog.svelte';
	import SuppliersTable from './components/suppliers-table.svelte';
	import { t } from '$lib/i18n/locale.svelte.js';

	const pageSizeOptions = [10, 25, 50, 100];
	const defaultPagination: SupplierPagination = {
		page: 1,
		pageSize: 10,
		totalRecords: 0,
		totalPages: 0
	};

	let suppliers = $state<Supplier[]>([]);
	let pagination = $state<SupplierPagination>(defaultPagination);
	let isLoading = $state(true);
	let errorMessage = $state<string | null>(null);
	const initialQuery = parseQuery(page.url.searchParams);
	let query = $state<Required<SupplierListParams>>(initialQuery);
	let searchValue = $state(initialQuery.search);
	let isCreateOpen = $state(false);
	let latestRequestId = 0;
	let searchTimer: ReturnType<typeof setTimeout> | undefined;

	onMount(() => {
		void loadSuppliers(query);
	});

	onDestroy(() => {
		if (searchTimer) clearTimeout(searchTimer);
	});

	async function loadSuppliers(params: SupplierListParams) {
		const requestId = ++latestRequestId;
		isLoading = true;
		errorMessage = null;

		try {
			const result = await listSuppliers(params);
			if (requestId !== latestRequestId) return;

			if (
				result.pagination.totalPages > 0 &&
				result.pagination.page > result.pagination.totalPages
			) {
				query = { ...query, page: result.pagination.totalPages };
				syncQueryUrl(query);
				await loadSuppliers(query);
				return;
			}

			suppliers = result.suppliers;
			pagination = result.pagination;
		} catch (error) {
			if (requestId !== latestRequestId) return;
			errorMessage = getErrorMessage(error);
			suppliers = [];
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
		void loadSuppliers(query);
	}

	function updatePage(nextPage: number) {
		const page = Math.max(1, Math.trunc(nextPage));
		if (page === query.page) return;

		query = { ...query, page };
		syncQueryUrl(query);
		void loadSuppliers(query);
	}

	function updateSearchValue(value: string) {
		searchValue = value;
		if (searchTimer) clearTimeout(searchTimer);

		searchTimer = setTimeout(() => {
			query = { ...query, search: value.trim(), page: 1 };
			syncQueryUrl(query);
			void loadSuppliers(query);
		}, 350);
	}

	function updateSupplierInList(updatedSupplier: Supplier) {
		suppliers = suppliers.map((supplier) =>
			supplier.id === updatedSupplier.id ? updatedSupplier : supplier
		);
	}

	async function refreshSuppliersAfterDelete() {
		await loadSuppliers(query);
	}

	async function refreshSuppliersAfterCreate() {
		query = { ...query, page: 1 };
		syncQueryUrl(query);
		await loadSuppliers(query);
	}

	function openCreateDialog() {
		isCreateOpen = true;
	}

	function parseQuery(searchParams: URLSearchParams): Required<SupplierListParams> {
		const page = Number(searchParams.get('page') ?? defaultPagination.page);
		const limit = Number(searchParams.get('limit') ?? defaultPagination.pageSize);
		const search = searchParams.get('search') ?? '';

		return {
			page: Number.isFinite(page) ? Math.max(1, Math.trunc(page)) : defaultPagination.page,
			limit: pageSizeOptions.includes(limit) ? limit : defaultPagination.pageSize,
			search
		};
	}

	function syncQueryUrl(nextQuery: Required<SupplierListParams>) {
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

		if (nextQuery.search) {
			nextUrl.searchParams.set('search', nextQuery.search);
		} else {
			nextUrl.searchParams.delete('search');
		}

		replaceState(resolve(`${nextUrl.pathname}${nextUrl.search}` as '/'), page.state);
	}

	function getErrorMessage(error: unknown) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return t('master.suppliers.errorLoad');
	}
</script>

<svelte:head>
	<title>Suppliers | PointFlow</title>
</svelte:head>

<div class="flex flex-col gap-6">
	<Card.Root>
		<Card.Header>
			<div>
				<Card.Title>{t('master.suppliers.title')}</Card.Title>
				<Card.Description>{t('master.suppliers.subtitle')}</Card.Description>
			</div>
			<Card.Action>
				<Button class="w-full sm:w-auto" onclick={openCreateDialog}>
					<PlusIcon />
					{t('master.suppliers.addBtn')}
				</Button>
			</Card.Action>
		</Card.Header>

		<Card.Content class="flex flex-col gap-3">
			<SearchField
				value={searchValue}
				disabled={isLoading}
				ariaLabel={t('master.suppliers.searchAriaLabel')}
				placeholder={t('master.suppliers.searchPlaceholder')}
				onValueChange={updateSearchValue}
			/>

			{#if errorMessage && !isLoading}
				<ErrorAlert title={t('master.suppliers.errorLoad')} message={errorMessage} />
			{:else}
				<SuppliersTable
					{suppliers}
					{isLoading}
					onSupplierDeleted={refreshSuppliersAfterDelete}
					onSupplierUpdated={updateSupplierInList}
					emptyMessage={query.search
						? t('master.suppliers.emptySearch')
						: t('master.suppliers.emptyDefault')}
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

	<SupplierCreateDialog bind:open={isCreateOpen} onCreated={refreshSuppliersAfterCreate} />
</div>
