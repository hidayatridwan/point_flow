<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { onMount } from 'svelte';
	import {
		fetchMemberProfile,
		logoutMember,
		memberAuthState,
		updateMemberPassword
	} from '$lib/api/member-auth.svelte.js';
	import { ApiError } from '$lib/api/types.js';
	import { formatDateTime } from '$lib/formatters.js';
	import { t } from '$lib/i18n/locale.svelte.js';

	let isLoadingProfile = $state(true);
	let profileError = $state<string | null>(null);
	let currentPassword = $state('');
	let newPassword = $state('');
	let isChangingPassword = $state(false);
	let passwordError = $state<string | null>(null);
	let passwordDone = $state<string | null>(null);
	let isLoggingOut = $state(false);

	onMount(() => {
		void loadProfile();
	});

	async function loadProfile() {
		isLoadingProfile = true;
		profileError = null;
		try {
			await fetchMemberProfile();
		} catch (error) {
			profileError = toFriendlyError(error, t('shop.profileLoadFailed'));
			if (error instanceof ApiError && error.status === 401) {
				await goto(resolve('/shop/account/login'));
				return;
			}
		} finally {
			isLoadingProfile = false;
		}
	}

	async function handleLogout() {
		if (isLoggingOut) return;
		isLoggingOut = true;
		try {
			await logoutMember();
		} finally {
			isLoggingOut = false;
		}
		await goto(resolve('/shop'));
	}

	async function handleChangePassword(event: SubmitEvent) {
		event.preventDefault();
		if (isChangingPassword || !currentPassword || newPassword.length < 8) return;
		isChangingPassword = true;
		passwordError = null;
		passwordDone = null;
		try {
			await updateMemberPassword(currentPassword, newPassword);
			passwordDone = t('shop.passwordChanged');
			currentPassword = '';
			newPassword = '';
			// Sesi ikut dihapus backend: arahkan masuk kembali.
			await goto(resolve('/shop/account/login'));
		} catch (error) {
			passwordError = toFriendlyError(error, t('shop.profileLoadFailed'));
		} finally {
			isChangingPassword = false;
		}
	}

	function toFriendlyError(error: unknown, fallback: string): string {
		if (error instanceof ApiError) {
			if (error.status === 429) return t('shop.rateLimited');
			return error.message;
		}
		if (error instanceof Error) return error.message;
		return fallback;
	}
</script>

<svelte:head>
	<title>{t('shop.profileTitle')} | PointFlow</title>
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
		<h1 class="text-xl font-bold text-[#333333]">{t('shop.profileTitle')}</h1>
	</div>

	{#if isLoadingProfile}
		<p class="rounded-2xl bg-white p-4 text-sm text-neutral-500 shadow-sm">
			{t('shop.loadFailed')}
		</p>
	{:else if profileError && !memberAuthState.member}
		<p role="alert" class="rounded-2xl bg-red-50 p-3 text-sm text-red-700">{profileError}</p>
	{:else if memberAuthState.member}
		{@const member = memberAuthState.member}
		<section class="mb-4 rounded-2xl bg-white p-4 shadow-sm">
			<p class="text-base font-bold text-[#333333]">{member.full_name}</p>
			<p class="mt-0.5 text-sm text-neutral-500">{member.member_no} · {member.phone}</p>
			{#if member.email}
				<p class="mt-0.5 text-sm text-neutral-500">{member.email}</p>
			{/if}
			<p class="mt-2 text-xs text-neutral-500">
				{t('shop.memberSince')}: {formatDateTime(member.joined_at)}
			</p>
			<button
				type="button"
				onclick={() => void handleLogout()}
				disabled={isLoggingOut}
				class="mt-4 w-full rounded-full border border-neutral-200 py-3 text-sm font-bold text-[#333333] transition-colors hover:bg-neutral-50 disabled:opacity-60"
			>
				{isLoggingOut ? t('shop.loggingOut') : t('shop.logoutButton')}
			</button>
		</section>

		<form
			onsubmit={(event) => {
				event.preventDefault();
				void handleChangePassword(event);
			}}
			class="rounded-2xl bg-white p-4 shadow-sm"
		>
			<h2 class="mb-2 text-sm font-bold text-[#333333]">{t('shop.changePasswordTitle')}</h2>
			<label class="mb-1 block text-xs font-semibold text-[#333333]" for="member-current-password">
				{t('shop.currentPasswordLabel')}
			</label>
			<input
				id="member-current-password"
				type="password"
				bind:value={currentPassword}
				required
				autocomplete="current-password"
				class="w-full rounded-xl border border-neutral-200 bg-white px-3 py-2.5 text-sm text-[#333333] outline-none placeholder:text-neutral-400 focus:border-[#ff6534]"
			/>
			<label class="mt-3 mb-1 block text-xs font-semibold text-[#333333]" for="member-new-password">
				{t('shop.newPasswordLabel')}
			</label>
			<input
				id="member-new-password"
				type="password"
				bind:value={newPassword}
				required
				minlength={8}
				autocomplete="new-password"
				placeholder={t('shop.newPasswordHint')}
				class="w-full rounded-xl border border-neutral-200 bg-white px-3 py-2.5 text-sm text-[#333333] outline-none placeholder:text-neutral-400 focus:border-[#ff6534]"
			/>

			{#if passwordError}
				<p role="alert" class="mt-3 rounded-2xl bg-red-50 p-3 text-sm text-red-700">
					{passwordError}
				</p>
			{/if}
			{#if passwordDone}
				<p class="mt-3 rounded-2xl bg-emerald-50 p-3 text-sm text-emerald-700">{passwordDone}</p>
			{/if}

			<button
				type="submit"
				disabled={isChangingPassword || !currentPassword || newPassword.length < 8}
				class="mt-4 w-full rounded-full bg-[#333333] py-3 text-sm font-bold text-white transition-colors hover:bg-black disabled:cursor-not-allowed disabled:opacity-60"
			>
				{isChangingPassword ? t('shop.changingPassword') : t('shop.changePasswordButton')}
			</button>
		</form>
	{/if}
</div>
