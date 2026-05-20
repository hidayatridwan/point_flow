<script lang="ts">
	import { page } from '$app/state';
	import { onDestroy, onMount } from 'svelte';
	import { CircleAlert } from '@lucide/svelte';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import {
		listProductCategories,
		type ProductCategory,
		type ProductCategoryListParams,
		type ProductCategoryPagination
	} from '$lib/api/product-categories.js';
	import { ApiError } from '$lib/api/types.js';
	import * as Alert from '$lib/components/ui/alert/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import ProductCategoryCreateDialog from './components/product-category-create-dialog.svelte';
	import ProductCategoriesPagination from './components/product-categories-pagination.svelte';
	import ProductCategoriesSearch from './components/product-categories-search.svelte';
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

	function updatePageSize(value: string) {
		const nextLimit = Number(value);
		if (!pageSizeOptions.includes(nextLimit)) return;

		query = { ...query, limit: nextLimit, page: 1 };
		void loadProductCategories(query);
	}

	function updatePage(nextPage: number) {
		const page = Math.max(1, Math.trunc(nextPage));
		if (page === query.page) return;

		query = { ...query, page };
		void loadProductCategories(query);
	}

	function updateSearchValue(value: string) {
		searchValue = value;
		if (searchTimer) clearTimeout(searchTimer);

		searchTimer = setTimeout(() => {
			query = { ...query, search: value.trim(), page: 1 };
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
	<div class="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
		<div class="flex flex-col gap-1">
			<h1 class="text-xl font-semibold tracking-tight">Product Categories</h1>
			<p class="max-w-2xl text-sm text-muted-foreground">
				Kelola pengelompokan produk agar pencatatan barang, pencarian item, dan laporan penjualan
				lebih mudah dibaca.
			</p>
		</div>
		<Button class="w-full sm:w-auto" onclick={openCreateDialog}>
			<PlusIcon />
			Add Category
		</Button>
	</div>

	<section class="flex flex-col gap-3">
		<ProductCategoriesSearch
			value={searchValue}
			disabled={isLoading}
			onValueChange={updateSearchValue}
		/>

		{#if errorMessage && !isLoading}
			<Alert.Root variant="destructive">
				<CircleAlert />
				<Alert.Title>Gagal memuat product categories.</Alert.Title>
				<Alert.Description>{errorMessage}</Alert.Description>
			</Alert.Root>
		{:else}
			<ProductCategoriesTable
				{categories}
				{isLoading}
				onCategoryDeleted={refreshCategoriesAfterDelete}
				onCategoryUpdated={updateCategoryInList}
				emptyMessage={query.search
					? 'Tidak ada kategori produk yang cocok dengan pencarian.'
					: 'Belum ada kategori produk.'}
			/>
			{#if !isLoading}
				<ProductCategoriesPagination
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
