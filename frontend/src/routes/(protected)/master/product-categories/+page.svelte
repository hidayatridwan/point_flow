<script lang="ts">
	import { replaceState } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import { onDestroy, onMount } from 'svelte';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import {
		listProductCategories,
		type ProductCategory,
		type ProductCategoryListParams,
		type ProductCategoryPagination
	} from '$lib/api/product-categories.js';
	import { ApiError } from '$lib/api/types.js';
	import DataPagination from '$lib/components/data-pagination.svelte';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import PageHeader from '$lib/components/page-header.svelte';
	import SearchField from '$lib/components/search-field.svelte';
	import { Button } from '$lib/components/ui/button/index.js';
	import ProductCategoryCreateDialog from './components/product-category-create-dialog.svelte';
	import ProductCategoriesTable from './components/product-categories-table.svelte';

	const pageSizeOptions = [10, 25, 50, 100];
	const defaultPagination: ProductCategoryPagination = {
		page: 1,
		pageSize: 10,
		totalRecords: 0,
		totalPages: 0
	};

	let categories = $state<ProductCategory[]>([]);
	let pagination = $state<ProductCategoryPagination>(defaultPagination);
	let isLoading = $state(true);
	let errorMessage = $state<string | null>(null);
	const initialQuery = parseProductCategoryQuery(page.url.searchParams);
	let query = $state<Required<ProductCategoryListParams>>(initialQuery);
	let searchValue = $state(initialQuery.search);
	let isCreateOpen = $state(false);
	let latestRequestId = 0;
	let searchTimer: ReturnType<typeof setTimeout> | undefined;

	onMount(() => {
		void loadProductCategories(query);
	});

	onDestroy(() => {
		if (searchTimer) clearTimeout(searchTimer);
	});

	async function loadProductCategories(params: ProductCategoryListParams) {
		const requestId = ++latestRequestId;
		isLoading = true;
		errorMessage = null;

		try {
			const result = await listProductCategories(params);
			if (requestId !== latestRequestId) return;

			if (
				result.pagination.totalPages > 0 &&
				result.pagination.page > result.pagination.totalPages
			) {
				query = { ...query, page: result.pagination.totalPages };
				syncProductCategoryQueryUrl(query);
				await loadProductCategories(query);
				return;
			}

			categories = result.categories;
			pagination = result.pagination;
		} catch (error) {
			if (requestId !== latestRequestId) return;
			errorMessage = getErrorMessage(error);
			categories = [];
		} finally {
			if (requestId === latestRequestId) {
				isLoading = false;
			}
		}
	}

	function updatePageSize(pageSize: number) {
		if (!pageSizeOptions.includes(pageSize)) return;

		query = { ...query, limit: pageSize, page: 1 };
		syncProductCategoryQueryUrl(query);
		void loadProductCategories(query);
	}

	function updatePage(nextPage: number) {
		const page = Math.max(1, Math.trunc(nextPage));
		if (page === query.page) return;

		query = { ...query, page };
		syncProductCategoryQueryUrl(query);
		void loadProductCategories(query);
	}

	function updateSearchValue(value: string) {
		searchValue = value;
		if (searchTimer) clearTimeout(searchTimer);

		searchTimer = setTimeout(() => {
			query = { ...query, search: value.trim(), page: 1 };
			syncProductCategoryQueryUrl(query);
			void loadProductCategories(query);
		}, 350);
	}

	function updateCategoryInList(updatedCategory: ProductCategory) {
		categories = categories.map((category) =>
			category.id === updatedCategory.id ? updatedCategory : category
		);
	}

	async function refreshCategoriesAfterDelete() {
		await loadProductCategories(query);
	}

	async function refreshCategoriesAfterCreate() {
		query = { ...query, page: 1 };
		syncProductCategoryQueryUrl(query);
		await loadProductCategories(query);
	}

	function openCreateDialog() {
		isCreateOpen = true;
	}

	function parseProductCategoryQuery(
		searchParams: URLSearchParams
	): Required<ProductCategoryListParams> {
		const page = Number(searchParams.get('page') ?? defaultPagination.page);
		const limit = Number(searchParams.get('limit') ?? defaultPagination.pageSize);
		const search = searchParams.get('search') ?? '';

		return {
			page: Number.isFinite(page) ? Math.max(1, Math.trunc(page)) : defaultPagination.page,
			limit: pageSizeOptions.includes(limit) ? limit : defaultPagination.pageSize,
			search
		};
	}

	function syncProductCategoryQueryUrl(nextQuery: Required<ProductCategoryListParams>) {
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
		return 'Failed to load product categories.';
	}
</script>

<svelte:head>
	<title>Product Categories | POS System</title>
</svelte:head>

<div class="flex flex-col gap-6">
	<PageHeader
		title="Product Categories"
		description="Manage product grouping so inventory records, item search, and sales reports stay easy to read."
	>
		{#snippet actions()}
			<Button class="w-full sm:w-auto" onclick={openCreateDialog}>
				<PlusIcon />
				Add Category
			</Button>
		{/snippet}
	</PageHeader>

	<section class="flex flex-col gap-3" aria-labelledby="product-categories-table-title">
		<h2 id="product-categories-table-title" class="sr-only">Product category records</h2>

		<SearchField
			value={searchValue}
			disabled={isLoading}
			ariaLabel="Search product categories"
			placeholder="Search product categories..."
			onValueChange={updateSearchValue}
		/>

		{#if errorMessage && !isLoading}
			<ErrorAlert title="Failed to load product categories." message={errorMessage} />
		{:else}
			<ProductCategoriesTable
				{categories}
				{isLoading}
				onCategoryDeleted={refreshCategoriesAfterDelete}
				onCategoryUpdated={updateCategoryInList}
				emptyMessage={query.search
					? 'No product categories match your search.'
					: 'No product categories yet.'}
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

	<ProductCategoryCreateDialog bind:open={isCreateOpen} onCreated={refreshCategoriesAfterCreate} />
</div>
