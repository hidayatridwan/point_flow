<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import CartBar from '$lib/components/storefront/cart-bar.svelte';
	import QtyStepper from '$lib/components/storefront/qty-stepper.svelte';
	import {
		cartCount,
		cartEstimate,
		cartState,
		removeLine,
		setLineQuantity
	} from '$lib/cart.svelte.js';
	import { formatIdr } from '$lib/formatters.js';
	import { t } from '$lib/i18n/locale.svelte.js';

	function goBack() {
		void goto(resolve('/shop'));
	}

	function goCheckout() {
		void goto(resolve('/shop/checkout'));
	}
</script>

<svelte:head>
	<title>{t('shop.cartTitle')} | PointFlow</title>
</svelte:head>

<div class="mx-auto max-w-xl px-4 pt-12 pb-32">
	<div class="mb-4 flex items-center gap-2">
		<button
			type="button"
			onclick={goBack}
			aria-label={t('shop.backToShop')}
			class="flex size-9 items-center justify-center rounded-full bg-white text-lg font-bold text-[#333333] shadow-sm"
		>
			‹
		</button>
		<h1 class="text-xl font-bold text-[#333333]">{t('shop.cartTitle')}</h1>
	</div>

	{#if cartState.lines.length === 0}
		<div class="rounded-2xl bg-white p-8 text-center shadow-sm">
			<p class="font-semibold text-[#333333]">{t('shop.cartEmpty')}</p>
			<p class="mt-1 text-sm text-neutral-500">{t('shop.cartEmptyHint')}</p>
			<button
				type="button"
				onclick={goBack}
				class="mt-4 rounded-full bg-[#ff6534] px-6 py-2.5 text-sm font-bold text-white"
			>
				{t('shop.backToShop')}
			</button>
		</div>
	{:else}
		<ul class="space-y-3">
			{#each cartState.lines as line (line.product_sku_id)}
				<li class="flex gap-3 rounded-2xl bg-white p-3 shadow-sm">
					{#if line.photo_url}
						<img
							src={line.photo_url}
							alt={line.product_name}
							loading="lazy"
							class="size-16 shrink-0 rounded-xl bg-neutral-100 object-cover"
						/>
					{:else}
						<div
							class="flex size-16 shrink-0 items-center justify-center rounded-xl bg-[#ffe9e1] text-xl font-bold text-[#ff6534]"
							aria-hidden="true"
						>
							{line.product_name.charAt(0).toUpperCase()}
						</div>
					{/if}
					<div class="min-w-0 flex-1">
						<p class="truncate text-sm font-semibold text-[#333333]">{line.product_name}</p>
						<p class="text-xs text-neutral-500">
							{formatIdr(line.unit_price)} / {line.unit}
						</p>
						<div class="mt-2 flex items-center justify-between">
							<QtyStepper
								quantity={line.quantity}
								max={line.stock > 0 ? line.stock : undefined}
								onchange={(quantity) => setLineQuantity(line.product_sku_id, quantity)}
							/>
							<button
								type="button"
								onclick={() => removeLine(line.product_sku_id)}
								class="text-xs font-semibold text-red-600"
							>
								{t('shop.remove')}
							</button>
						</div>
					</div>
					<p class="shrink-0 text-sm font-bold text-[#333333]">
						{formatIdr(line.quantity * line.unit_price)}
					</p>
				</li>
			{/each}
		</ul>
		<p class="mt-3 text-xs text-neutral-500">
			{t('shop.estimatedSubtotal')}: {formatIdr(cartEstimate())}
		</p>
	{/if}
</div>

<CartBar
	count={cartCount()}
	total={cartEstimate()}
	actionLabel={t('shop.goToCheckout')}
	onaction={goCheckout}
/>
