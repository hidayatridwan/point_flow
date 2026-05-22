<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import type { PathnameWithSearchOrHash } from '$app/types';
	import { page } from '$app/state';
	import { onMount } from 'svelte';
	import StoreIcon from '@lucide/svelte/icons/store';
	import { getAccessToken } from '$lib/api/auth.svelte.js';
	import { refreshAccessToken } from '$lib/api/client.js';
	import { login, type LoginCredentials } from '$lib/api/session.js';
	import { ApiError } from '$lib/api/types.js';
	import LoginForm from '$lib/components/login-form.svelte';
	import SessionTransition from '$lib/components/session-transition.svelte';

	const AUTH_TRANSITION_DELAY_MS = 700;
	const DEFAULT_REDIRECT_TO = '/dashboard';

	let isCheckingSession = $state(true);
	let isSubmitting = $state(false);
	let errorMessage = $state<string | null>(null);
	let transitionMessage = $state<string | null>(null);
	const redirectTo = $derived(getRedirectTo(page.url.searchParams.get('redirectTo')));

	onMount(() => {
		void redirectAuthenticatedSession();
	});

	async function redirectAuthenticatedSession() {
		try {
			const accessToken = getAccessToken() ?? (await refreshAccessToken());
			if (accessToken) {
				await goto(resolve(redirectTo as PathnameWithSearchOrHash), { replaceState: true });
			}
		} finally {
			isCheckingSession = false;
		}
	}

	async function handleLogin(credentials: LoginCredentials) {
		isSubmitting = true;
		errorMessage = null;
		transitionMessage = 'Verifying account...';

		try {
			await login(credentials);
			transitionMessage = 'Opening dashboard...';
			await wait(AUTH_TRANSITION_DELAY_MS);
			await goto(resolve(redirectTo as PathnameWithSearchOrHash), { replaceState: true });
		} catch (error) {
			transitionMessage = null;
			errorMessage = getLoginErrorMessage(error);
		} finally {
			isSubmitting = false;
		}
	}

	function getLoginErrorMessage(error: unknown) {
		if (error instanceof ApiError) {
			if (error.status === 401) return 'Invalid email or password.';
			return error.message;
		}
		if (error instanceof Error) return error.message;
		return 'Login failed. Please try again later.';
	}

	function getRedirectTo(value: string | null) {
		if (!value || !value.startsWith('/') || value === '/' || value.startsWith('//')) {
			return DEFAULT_REDIRECT_TO;
		}

		return value;
	}

	function wait(ms: number) {
		return new Promise((resolve) => setTimeout(resolve, ms));
	}
</script>

<svelte:head>
	<title>Sign In | POS System</title>
</svelte:head>

{#if isCheckingSession || transitionMessage}
	<SessionTransition
		class="min-h-svh bg-muted p-6 md:p-10"
		title={transitionMessage ?? 'Checking session...'}
		description="Please wait a moment."
	/>
{:else}
	<div class="flex min-h-svh flex-col items-center justify-center gap-6 bg-muted p-6 md:p-10">
		<div class="flex w-full max-w-sm flex-col gap-6">
			<a href={resolve('/')} class="flex items-center gap-2 self-center font-medium">
				<div
					class="flex size-6 items-center justify-center rounded-md bg-primary text-primary-foreground"
				>
					<StoreIcon class="size-4" />
				</div>
				POS System
			</a>
			<LoginForm {isSubmitting} {errorMessage} onSubmit={handleLogin} />
		</div>
	</div>
{/if}
