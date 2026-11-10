<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { resolve } from '$app/paths';
	import { onMount } from 'svelte';
	import { getPublicStoreInfo, type CheckoutResult, type StoreInfo } from '$lib/api/storefront.js';
	import { formatIdr } from '$lib/formatters.js';
	import { t } from '$lib/i18n/locale.svelte.js';
	import { loadMidtransSnap } from '$lib/midtrans-snap.js';

	let receipt = $state<CheckoutResult | null>(null);
	let storeInfo = $state<StoreInfo | null>(null);
	let copied = $state(false);
	let isPaying = $state(false);
	let snapMessage = $state<string | null>(null);
	let snapError = $state<string | null>(null);

	const snapToken = $derived(receipt?.snap_token ?? null);
	const isMidtransOrder = $derived(receipt?.payment_method === 'midtrans');

	const orderNo = $derived(page.params.order_no ?? '');

	onMount(() => {
		try {
			const raw = window.sessionStorage.getItem(`pointflow:receipt:${orderNo}`);
			if (raw) receipt = JSON.parse(raw) as CheckoutResult;
		} catch {
			receipt = null;
		}
		if (receipt) void loadStoreInfo();
	});

	async function loadStoreInfo() {
		try {
			storeInfo = await getPublicStoreInfo();
		} catch {
			storeInfo = null;
		}
	}

	function copyOrderNo() {
		if (!orderNo) return;
		try {
			void navigator.clipboard.writeText(orderNo).then(() => {
				copied = true;
				setTimeout(() => (copied = false), 2000);
			});
		} catch {
			// Clipboard tidak tersedia: nomor tetap terlihat untuk disalin manual.
		}
	}

	/** Buka Snap untuk token tersimpan (bayar maupun lihat status berjalan). */
	async function openSnap(statusCheck: boolean) {
		const token = snapToken;
		if (!token) return;
		isPaying = true;
		snapError = null;
		if (statusCheck) snapMessage = t('shop.checkingStatus');
		try {
			await loadMidtransSnap();
			window.snap?.pay(token, {
				onSuccess: () => {
					snapMessage = t('shop.midtransPaid');
				},
				onPending: () => {
					snapMessage = t('shop.midtransPending');
				},
				onError: () => {
					snapMessage = t('shop.midtransFailed');
				},
				onClose: () => {
					if (!snapMessage) snapMessage = t('shop.midtransClosed');
				}
			});
		} catch {
			snapError = t('shop.snapUnavailable');
		} finally {
			isPaying = false;
		}
	}

	function contactShop() {
		const whatsapp = storeInfo?.shop_whatsapp;
		if (!whatsapp) return;
		const text = encodeURIComponent(`Halo, saya mau konfirmasi pesanan ${orderNo}.`);
		window.open(`https://wa.me/${whatsapp}?text=${text}`, '_blank');
	}

	function orderAgain() {
		void goto(resolve('/shop'));
	}
</script>

<svelte:head>
	<title>{t('shop.successTitle')} | PointFlow</title>
</svelte:head>

<div class="mx-auto max-w-xl px-4 pt-12 pb-10">
	{#if !receipt}
		<div class="rounded-2xl bg-white p-8 text-center shadow-sm">
			<p class="font-semibold text-[#333333]">{t('shop.receiptMissing')}</p>
			<p class="mt-1 text-sm text-neutral-500">{t('shop.receiptMissingHint')}</p>
			<button
				type="button"
				onclick={orderAgain}
				class="mt-4 rounded-full bg-[#ff6534] px-6 py-2.5 text-sm font-bold text-white"
			>
				{t('shop.backToShop')}
			</button>
		</div>
	{:else}
		<div class="rounded-2xl bg-white p-6 text-center shadow-sm">
			<div
				class="mx-auto flex size-14 items-center justify-center rounded-full bg-green-100 text-2xl font-bold text-green-700"
				aria-hidden="true"
			>
				✓
			</div>
			<h1 class="mt-3 text-xl font-bold text-[#333333]">{t('shop.successTitle')}</h1>
			<p class="mt-1 text-xs text-neutral-500">{t('shop.successSubtitle')}</p>
			<p class="mt-4 text-xs font-semibold tracking-wide text-neutral-500 uppercase">
				{t('shop.orderNumber')}
			</p>
			<p class="mt-1 text-2xl font-bold break-all text-[#333333]">{receipt.order_no}</p>
			<button
				type="button"
				onclick={copyOrderNo}
				class="mt-2 rounded-full bg-neutral-100 px-4 py-1.5 text-xs font-bold text-[#333333]"
			>
				{copied ? t('shop.copied') : t('shop.copy')}
			</button>
		</div>

		<div class="mt-4 rounded-2xl bg-white p-4 shadow-sm">
			<ul class="divide-y divide-neutral-100">
				{#each receipt.items as item (item.product_sku)}
					<li class="flex justify-between gap-3 py-2 text-sm">
						<span class="text-[#333333]">
							{item.product_name}
							<span class="text-neutral-500">× {item.quantity}</span>
						</span>
						<span class="shrink-0 font-semibold text-[#333333]">
							{formatIdr(item.line_total)}
						</span>
					</li>
				{/each}
			</ul>
			<div
				class="flex justify-between border-t border-neutral-100 pt-3 text-base font-bold text-[#333333]"
			>
				<span>{t('shop.estimatedTotal')}</span>
				<span>{formatIdr(receipt.total_amount)}</span>
			</div>
		</div>

		{#if isMidtransOrder && snapToken}
			<div class="mt-4 rounded-2xl bg-white p-4 shadow-sm">
				<h2 class="text-sm font-bold text-[#333333]">{t('shop.payOnline')}</h2>
				<p class="mt-1 text-xl font-bold text-[#ff6534]">{formatIdr(receipt.total_amount)}</p>
				<p class="mt-1 text-xs text-neutral-500">{t('shop.payOnlineHint')}</p>
				{#if snapMessage}
					<p class="mt-2 rounded-xl bg-neutral-50 p-2.5 text-sm text-[#333333]">{snapMessage}</p>
				{/if}
				{#if snapError}
					<p role="alert" class="mt-2 rounded-xl bg-red-50 p-2.5 text-sm text-red-700">
						{snapError}
					</p>
				{/if}
				<div class="mt-3 grid gap-2">
					<button
						type="button"
						disabled={isPaying}
						onclick={() => void openSnap(false)}
						class="w-full rounded-full bg-[#ff6534] py-3 text-sm font-bold text-white transition-colors hover:bg-[#e5551f] disabled:cursor-not-allowed disabled:opacity-60"
					>
						{isPaying ? t('shop.paying') : t('shop.payNow')}
					</button>
					<button
						type="button"
						disabled={isPaying}
						onclick={() => void openSnap(true)}
						class="w-full rounded-full bg-white py-3 text-sm font-bold text-[#333333] shadow-sm disabled:cursor-not-allowed disabled:opacity-60"
					>
						{isPaying ? t('shop.checkingStatus') : t('shop.checkStatus')}
					</button>
				</div>
			</div>
		{/if}

		{#if receipt.payment_method === 'transfer' && receipt.payment_info}
			<div class="mt-4 rounded-2xl bg-white p-4 shadow-sm">
				<h2 class="text-sm font-bold text-[#333333]">{t('shop.payableTotal')}</h2>
				<p class="mt-1 text-xl font-bold text-[#ff6534]">
					{formatIdr(receipt.payment_info.total_amount)}
				</p>
				{#if receipt.payment_info.bank_accounts}
					<p class="mt-3 text-xs font-semibold text-neutral-500">{t('shop.bankAccounts')}</p>
					<p class="text-sm whitespace-pre-line text-[#333333]">
						{receipt.payment_info.bank_accounts}
					</p>
				{/if}
				{#if receipt.payment_info.ewallet_numbers}
					<p class="mt-3 text-xs font-semibold text-neutral-500">{t('shop.ewallets')}</p>
					<p class="text-sm whitespace-pre-line text-[#333333]">
						{receipt.payment_info.ewallet_numbers}
					</p>
				{/if}
				{#if receipt.payment_info.qris_image_url}
					<p class="mt-3 text-xs font-semibold text-neutral-500">{t('shop.qrisImage')}</p>
					<img
						src={receipt.payment_info.qris_image_url}
						alt="QRIS"
						loading="lazy"
						class="mt-1 w-full max-w-55 rounded-xl border border-neutral-200"
					/>
					<button
						type="button"
						onclick={() => {
							const url = receipt?.payment_info?.qris_image_url;
							if (url) window.open(url, '_blank', 'noopener');
						}}
						class="mt-1 inline-block text-xs font-semibold text-[#ff6534]"
					>
						{t('shop.viewQris')}
					</button>
				{/if}
			</div>
		{/if}

		<div class="mt-4 rounded-2xl bg-white p-4 shadow-sm">
			<p class="text-sm text-[#333333]">
				{receipt.fulfillment_method === 'delivery' ? t('shop.deliveryNext') : t('shop.pickupNext')}
			</p>
		</div>

		{#if storeInfo?.shop_whatsapp}
			<button
				type="button"
				onclick={contactShop}
				class="mt-4 w-full rounded-full bg-[#25d366] py-3.5 text-sm font-bold text-white"
			>
				{t('shop.contactShop')}
			</button>
		{/if}
		<button
			type="button"
			onclick={orderAgain}
			class="mt-2 w-full rounded-full bg-white py-3.5 text-sm font-bold text-[#333333] shadow-sm"
		>
			{t('shop.orderAgain')}
		</button>
	{/if}
</div>
