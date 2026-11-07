<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { onMount } from 'svelte';
	import CartBar from '$lib/components/storefront/cart-bar.svelte';
	import ProductCard from '$lib/components/storefront/product-card.svelte';
	import {
		getPublicCategories,
		getPublicProducts,
		getPublicStoreInfo,
		type PublicCategory,
		type PublicProduct,
		type StoreInfo
	} from '$lib/api/storefront.js';
	import { ApiError } from '$lib/api/types.js';
	import { addToCart, cartCount, cartEstimate } from '$lib/cart.svelte.js';
	import { t } from '$lib/i18n/locale.svelte.js';

	let storeInfo = $state<StoreInfo | null>(null);
	let categories = $state<PublicCategory[]>([]);
	let products = $state<PublicProduct[]>([]);
	let isLoading = $state(true);
	let errorMessage = $state<string | null>(null);
	let search = $state('');
	let activeCategoryId = $state<number | null>(null);

	let searchTimer: ReturnType<typeof setTimeout> | null = null;

	onMount(() => {
		void loadAll();
	});

	async function loadAll() {
		isLoading = true;
		errorMessage = null;
		try {
			const [info, cats] = await Promise.all([getPublicStoreInfo(), getPublicCategories()]);
			storeInfo = info;
			categories = cats;
			await loadProducts();
		} catch (error) {
			errorMessage = toFriendlyError(error);
		} finally {
			isLoading = false;
		}
	}

	async function loadProducts() {
		try {
			const result = await getPublicProducts({
				search: search.trim() || undefined,
				category_id: activeCategoryId ?? undefined,
				limit: 100
			});
			products = result.items;
		} catch (error) {
			errorMessage = toFriendlyError(error);
		}
	}

	function onSearchInput() {
		if (searchTimer) clearTimeout(searchTimer);
		searchTimer = setTimeout(() => void loadProducts(), 350);
	}

	function selectCategory(categoryId: number | null) {
		activeCategoryId = categoryId;
		void loadProducts();
	}

	function handleAdd(product: PublicProduct) {
		addToCart(
			{
				product_sku_id: product.product_sku_id,
				product_name: product.product_name,
				product_sku: product.product_sku,
				unit: product.unit,
				unit_price: product.selling_price,
				photo_url: product.photo_url,
				stock: product.stock
			},
			1
		);
	}

	function toFriendlyError(error: unknown): string {
		if (error instanceof ApiError) {
			if (error.status === 429) return t('shop.rateLimited');
			return error.message;
		}
		if (error instanceof Error) return error.message;
		return t('shop.loadFailed');
	}
</script>

<svelte:head>
	<title>{storeInfo?.store_name || 'Shop'} | PointFlow</title>
</svelte:head>

<div class="mx-auto max-w-xl px-4 pt-12 pb-32">
	{#if storeInfo?.store_name || storeInfo?.operating_hours}
		<header class="mb-4">
			{#if storeInfo?.store_name}
				<h1 class="text-2xl font-bold text-[#333333]">{storeInfo.store_name}</h1>
			{/if}
			{#if storeInfo?.operating_hours}
				<p class="mt-0.5 text-xs text-neutral-500">{storeInfo.operating_hours}</p>
			{/if}
		</header>
	{/if}

	<div class="relative mb-3">
		<svg
			class="pointer-events-none absolute top-1/2 left-4 size-4 -translate-y-1/2 text-neutral-400"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="2"
			aria-hidden="true"
		>
			<circle cx="11" cy="11" r="7" />
			<path d="m20 20-3.5-3.5" stroke-linecap="round" />
		</svg>
		<input
			type="search"
			bind:value={search}
			oninput={onSearchInput}
			placeholder={t('shop.searchPlaceholder')}
			aria-label={t('shop.searchPlaceholder')}
			class="w-full rounded-full bg-white py-3 pr-4 pl-11 text-sm text-[#333333] shadow-sm outline-none placeholder:text-neutral-400 focus:ring-2 focus:ring-[#ff6534]"
		/>
	</div>

	{#if categories.length > 0}
		<div class="mb-4 flex gap-2 overflow-x-auto pb-1" role="tablist">
			<button
				type="button"
				role="tab"
				aria-selected={activeCategoryId === null}
				onclick={() => selectCategory(null)}
				class="shrink-0 rounded-full px-3 py-1.5 text-xs font-semibold transition-colors"
				class:bg-[#333333]={activeCategoryId === null}
				class:text-white={activeCategoryId === null}
				class:bg-white={activeCategoryId !== null}
				class:text-neutral-600={activeCategoryId !== null}
			>
				{t('shop.allCategories')}
			</button>
			{#each categories as category (category.id)}
				<button
					type="button"
					role="tab"
					aria-selected={activeCategoryId === category.id}
					onclick={() => selectCategory(category.id)}
					class="shrink-0 rounded-full px-3 py-1.5 text-xs font-semibold transition-colors"
					class:bg-[#333333]={activeCategoryId === category.id}
					class:text-white={activeCategoryId === category.id}
					class:bg-white={activeCategoryId !== category.id}
					class:text-neutral-600={activeCategoryId !== category.id}
				>
					{category.name}
				</button>
			{/each}
		</div>
	{/if}

	{#if isLoading}
		<div class="columns-2 gap-4" aria-hidden="true">
			{#each [1, 2, 3, 4] as i (i)}
				<div class="mb-4 h-56 animate-pulse rounded-2xl bg-white"></div>
			{/each}
		</div>
	{:else if errorMessage && products.length === 0}
		<div class="rounded-2xl bg-white p-6 text-center shadow-sm">
			<p class="text-sm text-[#333333]">{errorMessage}</p>
			<button
				type="button"
				onclick={() => void loadAll()}
				class="mt-3 rounded-full bg-[#ff6534] px-5 py-2 text-sm font-bold text-white"
			>
				{t('common.tryAgain')}
			</button>
		</div>
	{:else}
		<div class="columns-2 gap-4">
			{#each products as product (product.product_sku_id)}
				<ProductCard {product} onadd={handleAdd} />
			{/each}
		</div>
	{/if}
</div>

<CartBar
	count={cartCount()}
	total={cartEstimate()}
	actionLabel={t('shop.cartTitle')}
	onaction={() => void goto(resolve('/shop/cart'))}
/>
