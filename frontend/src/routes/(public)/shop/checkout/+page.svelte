<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { onMount } from 'svelte';
	import { memberAuthState, submitMemberOrder } from '$lib/api/member-auth.svelte.js';
	import { getPublicStoreInfo, type StoreInfo } from '$lib/api/storefront.js';
	import { ApiError } from '$lib/api/types.js';
	import { cartEstimate, cartState, clearCart } from '$lib/cart.svelte.js';
	import { formatIdr } from '$lib/formatters.js';
	import { t } from '$lib/i18n/locale.svelte.js';

	type Fulfillment = 'pickup' | 'delivery';
	type Payment = 'pay_at_store' | 'transfer';

	let storeInfo = $state<StoreInfo | null>(null);
	let customerName = $state('');
	let customerPhone = $state('');
	let fulfillment = $state<Fulfillment>('pickup');
	let payment = $state<Payment>('pay_at_store');
	let deliveryAddress = $state('');
	let notes = $state('');
	let isSubmitting = $state(false);
	let errorMessage = $state<string | null>(null);
	let addressPrefilledFor: number | null = null;

	const deliveryFee = $derived(fulfillment === 'delivery' ? (storeInfo?.delivery_fee ?? 0) : 0);
	const total = $derived(cartEstimate() + deliveryFee);

	onMount(() => {
		if (cartState.lines.length === 0) {
			void goto(resolve('/shop/cart'));
			return;
		}
		void loadStoreInfo();
	});

	// Member login: alamat default dari profil (tetap bisa diedit per order).
	$effect(() => {
		const member = memberAuthState.member;
		if (!member) {
			addressPrefilledFor = null;
			return;
		}
		if (addressPrefilledFor !== member.id) {
			addressPrefilledFor = member.id;
			if (!deliveryAddress.trim() && member.address?.trim()) {
				deliveryAddress = member.address.trim();
			}
		}
	});

	async function loadStoreInfo() {
		try {
			storeInfo = await getPublicStoreInfo();
		} catch {
			// Info toko opsional di halaman ini; ongkir default 0 bila gagal dimuat.
		}
	}

	function validate(): string | null {
		// Identitas member diambil dari profil (readonly); guest wajib isi.
		if (!memberAuthState.member && (!customerName.trim() || !customerPhone.trim())) {
			return t('shop.checkoutFailed');
		}
		if (fulfillment === 'delivery' && !deliveryAddress.trim()) return t('shop.checkoutFailed');
		return null;
	}

	async function submit() {
		errorMessage = validate();
		if (errorMessage) return;
		isSubmitting = true;
		try {
			const member = memberAuthState.member;
			const result = await submitMemberOrder({
				customer_name: member ? member.full_name : customerName.trim(),
				customer_phone: member ? member.phone : customerPhone.trim(),
				fulfillment_method: fulfillment,
				delivery_address: fulfillment === 'delivery' ? deliveryAddress.trim() : undefined,
				payment_method: payment,
				notes: notes.trim() || undefined,
				items: cartState.lines.map((line) => ({
					product_sku_id: line.product_sku_id,
					quantity: line.quantity
				}))
			});
			try {
				window.sessionStorage.setItem(
					`pointflow:receipt:${result.order_no}`,
					JSON.stringify(result)
				);
			} catch {
				// Penyimpanan penuh: halaman sukses tetap bisa dirender sekali ini via state?
				// Tidak — tanpa receipt, arahkan dengan fallback yang jelas di halaman sukses.
			}
			clearCart();
			await goto(resolve(`/shop/success/${result.order_no}`));
		} catch (error) {
			if (error instanceof ApiError && error.status === 429) {
				errorMessage = t('shop.rateLimited');
			} else if (error instanceof Error) {
				errorMessage = error.message;
			} else {
				errorMessage = t('shop.checkoutFailed');
			}
		} finally {
			isSubmitting = false;
		}
	}

	function optionCard(selected: boolean): string {
		return selected ? 'border-[#ff6534] bg-[#fff3ee]' : 'border-neutral-200 bg-white';
	}
</script>

<svelte:head>
	<title>{t('shop.checkoutTitle')} | PointFlow</title>
</svelte:head>

<div class="mx-auto max-w-xl px-4 pt-12 pb-10">
	<div class="mb-4 flex items-center gap-2">
		<button
			type="button"
			onclick={() => void goto(resolve('/shop/cart'))}
			aria-label={t('shop.cartTitle')}
			class="flex size-9 items-center justify-center rounded-full bg-white text-lg font-bold text-[#333333] shadow-sm"
		>
			‹
		</button>
		<h1 class="text-xl font-bold text-[#333333]">{t('shop.checkoutTitle')}</h1>
	</div>

	<form
		onsubmit={(event) => {
			event.preventDefault();
			void submit();
		}}
		class="space-y-4"
	>
		<section class="rounded-2xl bg-white p-4 shadow-sm">
			{#if memberAuthState.member}
				{@const member = memberAuthState.member}
				<p class="text-sm font-bold text-[#333333]">
					{member.full_name} <span class="font-normal text-neutral-500">({member.member_no})</span>
				</p>
				<p class="mt-0.5 text-sm text-neutral-500">{member.phone}</p>
				<p class="mt-2 rounded-xl bg-emerald-50 p-2.5 text-xs text-emerald-700">
					{t('shop.checkoutAsMember')
						.replace('{name}', member.full_name)
						.replace('{memberNo}', member.member_no)}
				</p>
			{:else}
				<label class="mb-1 block text-xs font-semibold text-[#333333]" for="shop-name">
					{t('shop.customerName')}
				</label>
				<input
					id="shop-name"
					type="text"
					bind:value={customerName}
					required
					maxlength={120}
					placeholder={t('shop.customerNamePlaceholder')}
					autocomplete="name"
					class="w-full rounded-xl border border-neutral-200 bg-white px-3 py-2.5 text-sm text-[#333333] outline-none placeholder:text-neutral-400 focus:border-[#ff6534]"
				/>
				<label class="mt-3 mb-1 block text-xs font-semibold text-[#333333]" for="shop-phone">
					{t('shop.customerPhone')}
				</label>
				<input
					id="shop-phone"
					type="tel"
					bind:value={customerPhone}
					required
					maxlength={30}
					placeholder={t('shop.customerPhonePlaceholder')}
					autocomplete="tel"
					class="w-full rounded-xl border border-neutral-200 bg-white px-3 py-2.5 text-sm text-[#333333] outline-none placeholder:text-neutral-400 focus:border-[#ff6534]"
				/>
				<a
					href={resolve('/shop/account/login')}
					class="mt-3 block text-xs font-semibold text-[#ff6534] hover:underline"
				>
					{t('shop.loginPrompt')}
				</a>
			{/if}
		</section>

		<section class="rounded-2xl bg-white p-4 shadow-sm">
			<h2 class="mb-2 text-xs font-semibold text-[#333333]">{t('shop.fulfillment')}</h2>
			<div class="grid grid-cols-2 gap-2">
				<button
					type="button"
					onclick={() => (fulfillment = 'pickup')}
					aria-pressed={fulfillment === 'pickup'}
					class="rounded-2xl border-2 p-3 text-left transition-colors {optionCard(
						fulfillment === 'pickup'
					)}"
				>
					<p class="text-sm font-bold text-[#333333]">{t('shop.pickup')}</p>
					<p class="mt-0.5 text-[11px] text-neutral-500">{t('shop.pickupHint')}</p>
				</button>
				<button
					type="button"
					onclick={() => (fulfillment = 'delivery')}
					aria-pressed={fulfillment === 'delivery'}
					class="rounded-2xl border-2 p-3 text-left transition-colors {optionCard(
						fulfillment === 'delivery'
					)}"
				>
					<p class="text-sm font-bold text-[#333333]">{t('shop.delivery')}</p>
					<p class="mt-0.5 text-[11px] text-neutral-500">{t('shop.deliveryHint')}</p>
				</button>
			</div>
			{#if fulfillment === 'delivery'}
				<label class="mt-3 mb-1 block text-xs font-semibold text-[#333333]" for="shop-address">
					{t('shop.address')}
				</label>
				<textarea
					id="shop-address"
					bind:value={deliveryAddress}
					required
					rows="2"
					placeholder={t('shop.addressPlaceholder')}
					autocomplete="street-address"
					class="w-full rounded-xl border border-neutral-200 bg-white px-3 py-2.5 text-sm text-[#333333] outline-none placeholder:text-neutral-400 focus:border-[#ff6534]"
				></textarea>
			{/if}
		</section>

		<section class="rounded-2xl bg-white p-4 shadow-sm">
			<h2 class="mb-2 text-xs font-semibold text-[#333333]">{t('shop.payment')}</h2>
			<div class="grid grid-cols-2 gap-2">
				<button
					type="button"
					onclick={() => (payment = 'pay_at_store')}
					aria-pressed={payment === 'pay_at_store'}
					class="rounded-2xl border-2 p-3 text-left transition-colors {optionCard(
						payment === 'pay_at_store'
					)}"
				>
					<p class="text-sm font-bold text-[#333333]">{t('shop.payAtStore')}</p>
					<p class="mt-0.5 text-[11px] text-neutral-500">{t('shop.payAtStoreHint')}</p>
				</button>
				<button
					type="button"
					onclick={() => (payment = 'transfer')}
					aria-pressed={payment === 'transfer'}
					class="rounded-2xl border-2 p-3 text-left transition-colors {optionCard(
						payment === 'transfer'
					)}"
				>
					<p class="text-sm font-bold text-[#333333]">{t('shop.transfer')}</p>
					<p class="mt-0.5 text-[11px] text-neutral-500">{t('shop.transferHint')}</p>
				</button>
			</div>
		</section>

		<section class="rounded-2xl bg-white p-4 shadow-sm">
			<label class="mb-1 block text-xs font-semibold text-[#333333]" for="shop-notes">
				{t('shop.notes')}
			</label>
			<textarea
				id="shop-notes"
				bind:value={notes}
				rows="2"
				placeholder={t('shop.notesPlaceholder')}
				class="w-full rounded-xl border border-neutral-200 bg-white px-3 py-2.5 text-sm text-[#333333] outline-none placeholder:text-neutral-400 focus:border-[#ff6534]"
			></textarea>
			<dl class="mt-3 space-y-1 text-sm">
				<div class="flex justify-between text-neutral-600">
					<dt>{t('shop.estimatedSubtotal')}</dt>
					<dd>{formatIdr(cartEstimate())}</dd>
				</div>
				{#if fulfillment === 'delivery'}
					<div class="flex justify-between text-neutral-600">
						<dt>{t('shop.deliveryFee')}</dt>
						<dd>
							{#if deliveryFee > 0}
								{formatIdr(deliveryFee)}
							{:else}
								{t('shop.free')}
							{/if}
						</dd>
					</div>
				{/if}
				<div
					class="flex justify-between border-t border-neutral-100 pt-2 text-base font-bold text-[#333333]"
				>
					<dt>{t('shop.estimatedTotal')}</dt>
					<dd>{formatIdr(total)}</dd>
				</div>
			</dl>
		</section>

		{#if errorMessage}
			<p role="alert" class="rounded-2xl bg-red-50 p-3 text-sm text-red-700">{errorMessage}</p>
		{/if}

		<button
			type="submit"
			disabled={isSubmitting || cartState.lines.length === 0}
			class="w-full rounded-full bg-[#ff6534] py-3.5 text-sm font-bold text-white transition-colors hover:bg-[#e5551f] disabled:cursor-not-allowed disabled:opacity-60"
		>
			{isSubmitting ? t('shop.submittingOrder') : t('shop.submitOrder')}
		</button>
	</form>
</div>
