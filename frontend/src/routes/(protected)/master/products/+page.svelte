<script lang="ts">
	import { replaceState } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import { onDestroy, onMount } from 'svelte';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import {
		listProducts,
		type Product,
		type ProductListParams,
		type ProductPagination
	} from '$lib/api/products.js';
	import { ApiError } from '$lib/api/types.js';
	import DataPagination from '$lib/components/data-pagination.svelte';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import * as Card from '$lib/components/ui/card/index.js';
	import SearchField from '$lib/components/search-field.svelte';
	import { Button } from '$lib/components/ui/button/index.js';
	import ProductCreateDialog from './components/product-create-dialog.svelte';
	import ProductsTable from './components/products-table.svelte';
	import { t } from '$lib/i18n/locale.svelte.js';

	const pageSizeOptions = [10, 25, 50, 100];
	const defaultPagination: ProductPagination = {
		page: 1,
		pageSize: 10,
		totalRecords: 0,
		totalPages: 0
	};

	let products = $state<Product[]>([]);
	let pagination = $state<ProductPagination>(defaultPagination);
	let isLoading = $state(true);
	let errorMessage = $state<string | null>(null);
	const initialQuery = parseQuery(page.url.searchParams);
	let query = $state<Required<ProductListParams>>(initialQuery);
	let searchValue = $state(initialQuery.search);
	let isCreateOpen = $state(false);
	let latestRequestId = 0;
	let searchTimer: ReturnType<typeof setTimeout> | undefined;

	onMount(() => {
		void loadProducts(query);
	});

	onDestroy(() => {
		if (searchTimer) clearTimeout(searchTimer);
	});

	async function loadProducts(params: ProductListParams) {
		const requestId = ++latestRequestId;
		isLoading = true;
		errorMessage = null;

		try {
			const result = await listProducts(params);
			if (requestId !== latestRequestId) return;

			if (
				result.pagination.totalPages > 0 &&
				result.pagination.page > result.pagination.totalPages
			) {
				query = { ...query, page: result.pagination.totalPages };
				syncQueryUrl(query);
				await loadProducts(query);
				return;
			}

			products = result.products;
			pagination = result.pagination;
		} catch (error) {
			if (requestId !== latestRequestId) return;
			errorMessage = getErrorMessage(error);
			products = [];
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
		void loadProducts(query);
	}

	function updatePage(nextPage: number) {
		const page = Math.max(1, Math.trunc(nextPage));
		if (page === query.page) return;

		query = { ...query, page };
		syncQueryUrl(query);
		void loadProducts(query);
	}

	function updateSearchValue(value: string) {
		searchValue = value;
		if (searchTimer) clearTimeout(searchTimer);

		searchTimer = setTimeout(() => {
			query = { ...query, search: value.trim(), page: 1 };
			syncQueryUrl(query);
			void loadProducts(query);
		}, 350);
	}

	function updateProductInList(updatedProduct: Product) {
		products = products.map((product) =>
			product.id === updatedProduct.id ? updatedProduct : product
		);
	}

	async function refreshProductsAfterDelete() {
		await loadProducts(query);
	}

	async function refreshProductsAfterCreate() {
		query = { ...query, page: 1 };
		syncQueryUrl(query);
		await loadProducts(query);
	}

	function openCreateDialog() {
		isCreateOpen = true;
	}

	function parseQuery(searchParams: URLSearchParams): Required<ProductListParams> {
		const page = Number(searchParams.get('page') ?? defaultPagination.page);
		const limit = Number(searchParams.get('limit') ?? defaultPagination.pageSize);
		const search = searchParams.get('search') ?? '';

		return {
			page: Number.isFinite(page) ? Math.max(1, Math.trunc(page)) : defaultPagination.page,
			limit: pageSizeOptions.includes(limit) ? limit : defaultPagination.pageSize,
			search
		};
	}

	function syncQueryUrl(nextQuery: Required<ProductListParams>) {
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
		return t('master.products.errorLoad');
	}
</script>

<svelte:head>
	<title>Products | POS System</title>
</svelte:head>

<div class="flex flex-col gap-6">
	<Card.Root>
		<Card.Header>
			<div>
				<Card.Title>{t('master.products.title')}</Card.Title>
				<Card.Description>{t('master.products.subtitle')}</Card.Description>
			</div>
			<Card.Action>
				<Button class="w-full sm:w-auto" onclick={openCreateDialog}>
					<PlusIcon />
					{t('master.products.addBtn')}
				</Button>
			</Card.Action>
		</Card.Header>

		<Card.Content class="flex flex-col gap-3">
			<SearchField
				value={searchValue}
				disabled={isLoading}
				ariaLabel={t('master.products.searchAriaLabel')}
				placeholder={t('master.products.searchPlaceholder')}
				onValueChange={updateSearchValue}
			/>

			{#if errorMessage && !isLoading}
				<ErrorAlert title={t('master.products.errorLoad')} message={errorMessage} />
			{:else}
				<ProductsTable
					{products}
					{isLoading}
					onProductDeleted={refreshProductsAfterDelete}
					onProductUpdated={updateProductInList}
					emptyMessage={query.search
						? t('master.products.emptySearch')
						: t('master.products.emptyDefault')}
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

	<ProductCreateDialog bind:open={isCreateOpen} onCreated={refreshProductsAfterCreate} />
</div>
