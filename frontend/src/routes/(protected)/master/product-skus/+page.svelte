<script lang="ts">
	import { replaceState } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import { onDestroy, onMount } from 'svelte';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import {
		listProductSkus,
		type ProductSku,
		type ProductSkuListParams,
		type ProductSkuPagination
	} from '$lib/api/product-skus.js';
	import { ApiError } from '$lib/api/types.js';
	import DataPagination from '$lib/components/data-pagination.svelte';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import PageHeader from '$lib/components/page-header.svelte';
	import SearchField from '$lib/components/search-field.svelte';
	import { Button } from '$lib/components/ui/button/index.js';
	import ProductSkuCreateDialog from './components/product-sku-create-dialog.svelte';
	import ProductSkusTable from './components/product-skus-table.svelte';
	import { t } from '$lib/i18n/locale.svelte.js';

	const pageSizeOptions = [10, 25, 50, 100];
	const defaultPagination: ProductSkuPagination = {
		page: 1,
		pageSize: 10,
		totalRecords: 0,
		totalPages: 0
	};

	let productSkus = $state<ProductSku[]>([]);
	let pagination = $state<ProductSkuPagination>(defaultPagination);
	let isLoading = $state(true);
	let errorMessage = $state<string | null>(null);
	const initialQuery = parseQuery(page.url.searchParams);
	let query = $state<Required<ProductSkuListParams>>(initialQuery);
	let searchValue = $state(initialQuery.search);
	let isCreateOpen = $state(false);
	let latestRequestId = 0;
	let searchTimer: ReturnType<typeof setTimeout> | undefined;

	onMount(() => {
		void loadProductSkus(query);
	});

	onDestroy(() => {
		if (searchTimer) clearTimeout(searchTimer);
	});

	async function loadProductSkus(params: ProductSkuListParams) {
		const requestId = ++latestRequestId;
		isLoading = true;
		errorMessage = null;

		try {
			const result = await listProductSkus(params);
			if (requestId !== latestRequestId) return;

			if (
				result.pagination.totalPages > 0 &&
				result.pagination.page > result.pagination.totalPages
			) {
				query = { ...query, page: result.pagination.totalPages };
				syncQueryUrl(query);
				await loadProductSkus(query);
				return;
			}

			productSkus = result.productSkus;
			pagination = result.pagination;
		} catch (error) {
			if (requestId !== latestRequestId) return;
			errorMessage = getErrorMessage(error);
			productSkus = [];
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
		void loadProductSkus(query);
	}

	function updatePage(nextPage: number) {
		const page = Math.max(1, Math.trunc(nextPage));
		if (page === query.page) return;

		query = { ...query, page };
		syncQueryUrl(query);
		void loadProductSkus(query);
	}

	function updateSearchValue(value: string) {
		searchValue = value;
		if (searchTimer) clearTimeout(searchTimer);

		searchTimer = setTimeout(() => {
			query = { ...query, search: value.trim(), page: 1 };
			syncQueryUrl(query);
			void loadProductSkus(query);
		}, 350);
	}

	function updateProductSkuInList(updatedProductSku: ProductSku) {
		productSkus = productSkus.map((productSku) =>
			productSku.id === updatedProductSku.id ? updatedProductSku : productSku
		);
	}

	async function refreshProductSkusAfterDelete() {
		await loadProductSkus(query);
	}

	async function refreshProductSkusAfterCreate() {
		query = { ...query, page: 1 };
		syncQueryUrl(query);
		await loadProductSkus(query);
	}

	function openCreateDialog() {
		isCreateOpen = true;
	}

	function parseQuery(searchParams: URLSearchParams): Required<ProductSkuListParams> {
		const page = Number(searchParams.get('page') ?? defaultPagination.page);
		const limit = Number(searchParams.get('limit') ?? defaultPagination.pageSize);
		const search = searchParams.get('search') ?? '';

		return {
			page: Number.isFinite(page) ? Math.max(1, Math.trunc(page)) : defaultPagination.page,
			limit: pageSizeOptions.includes(limit) ? limit : defaultPagination.pageSize,
			search
		};
	}

	function syncQueryUrl(nextQuery: Required<ProductSkuListParams>) {
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
		return t('master.productSkus.errorLoad');
	}
</script>

<svelte:head>
	<title>Product SKUs | POS System</title>
</svelte:head>

<div class="flex flex-col gap-6">
	<PageHeader
		title={t('master.productSkus.title')}
		description={t('master.productSkus.subtitle')}
	>
		{#snippet actions()}
			<Button class="w-full sm:w-auto" onclick={openCreateDialog}>
				<PlusIcon />
				{t('master.productSkus.addBtn')}
			</Button>
		{/snippet}
	</PageHeader>

	<section class="flex flex-col gap-3" aria-labelledby="product-skus-table-title">
		<h2 id="product-skus-table-title" class="sr-only">{t('master.productSkus.srHeading')}</h2>

		<SearchField
			value={searchValue}
			disabled={isLoading}
			ariaLabel={t('master.productSkus.searchAriaLabel')}
			placeholder={t('master.productSkus.searchPlaceholder')}
			onValueChange={updateSearchValue}
		/>

		{#if errorMessage && !isLoading}
			<ErrorAlert title={t('master.productSkus.errorLoad')} message={errorMessage} />
		{:else}
			<ProductSkusTable
				{productSkus}
				{isLoading}
				onProductSkuDeleted={refreshProductSkusAfterDelete}
				onProductSkuUpdated={updateProductSkuInList}
				emptyMessage={query.search ? t('master.productSkus.emptySearch') : t('master.productSkus.emptyDefault')}
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

	<ProductSkuCreateDialog bind:open={isCreateOpen} onCreated={refreshProductSkusAfterCreate} />
</div>
