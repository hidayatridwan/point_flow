<script lang="ts">
	import { formatIdr } from '$lib/formatters.js';
	import { t } from '$lib/i18n/locale.svelte.js';
	import type { PublicProduct } from '$lib/api/storefront.js';

	type Props = {
		product: PublicProduct;
		onadd?: (product: PublicProduct) => void;
	};

	let { product, onadd }: Props = $props();

	const isOutOfStock = $derived(product.stock <= 0);
	const isLowStock = $derived(product.stock > 0 && product.stock <= 5);

	function stockLabel(): string {
		if (isOutOfStock) return t('shop.outOfStock');
		if (isLowStock) return t('shop.lowStock').replace('{count}', String(product.stock));
		return t('shop.inStock');
	}
</script>

<article
	class="mb-4 w-full break-inside-avoid overflow-hidden rounded-2xl bg-white shadow-sm"
	class:grayscale={isOutOfStock}
>
	{#if product.photo_url}
		<img
			src={product.photo_url}
			alt={product.product_name}
			loading="lazy"
			class="h-auto w-full bg-neutral-100 object-cover"
		/>
	{:else}
		<div
			class="flex aspect-square w-full items-center justify-center bg-[#ffe9e1] text-4xl font-bold text-[#ff6534]"
			aria-hidden="true"
		>
			{product.product_name.charAt(0).toUpperCase()}
		</div>
	{/if}
	<div class="relative p-3">
		{#if product.category_name}
			<p class="text-[11px] text-neutral-500">{product.category_name}</p>
		{/if}
		<h3 class="pr-10 text-sm font-semibold text-[#333333]">{product.product_name}</h3>
		<p class="text-xs text-neutral-500">{product.unit}</p>
		<p class="mt-1 text-base font-bold text-[#333333]">{formatIdr(product.selling_price)}</p>
		<p
			class="mt-0.5 text-xs"
			class:text-red-600={isOutOfStock}
			class:text-[#ff6534]={isLowStock}
			class:text-neutral-500={!isOutOfStock && !isLowStock}
		>
			{stockLabel()}
		</p>
		<button
			type="button"
			disabled={isOutOfStock}
			aria-label={t('shop.addToCart')}
			onclick={() => onadd?.(product)}
			class="absolute right-3 bottom-3 flex size-9 items-center justify-center rounded-full text-xl font-bold text-white transition-colors"
			class:bg-[#ff6534]={!isOutOfStock}
			class:hover:bg-[#e5551f]={!isOutOfStock}
			class:bg-neutral-300={isOutOfStock}
			class:cursor-not-allowed={isOutOfStock}
		>
			+
		</button>
	</div>
</article>
