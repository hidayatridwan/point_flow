<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { onMount } from 'svelte';
	import { loginMember, memberAuthState } from '$lib/api/member-auth.svelte.js';
	import { ApiError } from '$lib/api/types.js';
	import { t } from '$lib/i18n/locale.svelte.js';

	let phone = $state('');
	let password = $state('');
	let isSubmitting = $state(false);
	let errorMessage = $state<string | null>(null);

	onMount(() => {
		if (memberAuthState.member) {
			void goto(resolve('/shop/account/profile'));
		}
	});

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		if (isSubmitting || !phone.trim() || !password) return;
		isSubmitting = true;
		errorMessage = null;
		try {
			await loginMember(phone, password);
			await goto(resolve('/shop'));
		} catch (error) {
			errorMessage = toFriendlyError(error);
		} finally {
			isSubmitting = false;
		}
	}

	function toFriendlyError(error: unknown): string {
		if (error instanceof ApiError) {
			if (error.status === 401) return t('shop.loginFailed');
			if (error.status === 429) return t('shop.rateLimited');
			return error.message;
		}
		if (error instanceof Error) return error.message;
		return t('shop.loginFailed');
	}
</script>

<svelte:head>
	<title>{t('shop.accountTitle')} | PointFlow</title>
</svelte:head>

<div class="mx-auto max-w-xl px-4 pt-12 pb-10">
	<div class="mb-4 flex items-center gap-2">
		<button
			type="button"
			onclick={() => void goto(resolve('/shop'))}
			aria-label={t('shop.backToShop')}
			class="flex size-9 items-center justify-center rounded-full bg-white text-lg font-bold text-[#333333] shadow-sm"
		>
			‹
		</button>
		<h1 class="text-xl font-bold text-[#333333]">{t('shop.accountTitle')}</h1>
	</div>
	<p class="mb-4 text-sm text-neutral-500">{t('shop.accountSubtitle')}</p>

	<form
		onsubmit={(event) => {
			event.preventDefault();
			void submit(event);
		}}
		class="space-y-4"
	>
		<section class="rounded-2xl bg-white p-4 shadow-sm">
			<label class="mb-1 block text-xs font-semibold text-[#333333]" for="member-phone">
				{t('shop.phoneLabel')}
			</label>
			<input
				id="member-phone"
				type="tel"
				bind:value={phone}
				required
				maxlength={30}
				placeholder={t('shop.phonePlaceholder')}
				autocomplete="tel"
				class="w-full rounded-xl border border-neutral-200 bg-white px-3 py-2.5 text-sm text-[#333333] outline-none placeholder:text-neutral-400 focus:border-[#ff6534]"
			/>
			<label class="mt-3 mb-1 block text-xs font-semibold text-[#333333]" for="member-password">
				{t('shop.passwordLabel')}
			</label>
			<input
				id="member-password"
				type="password"
				bind:value={password}
				required
				placeholder={t('shop.passwordPlaceholder')}
				autocomplete="current-password"
				class="w-full rounded-xl border border-neutral-200 bg-white px-3 py-2.5 text-sm text-[#333333] outline-none placeholder:text-neutral-400 focus:border-[#ff6534]"
			/>
		</section>

		{#if errorMessage}
			<p role="alert" class="rounded-2xl bg-red-50 p-3 text-sm text-red-700">{errorMessage}</p>
		{/if}

		<button
			type="submit"
			disabled={isSubmitting || !phone.trim() || !password}
			class="w-full rounded-full bg-[#ff6534] py-3.5 text-sm font-bold text-white transition-colors hover:bg-[#e5551f] disabled:cursor-not-allowed disabled:opacity-60"
		>
			{isSubmitting ? t('shop.loggingIn') : t('shop.loginButton')}
		</button>
	</form>
</div>
