<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { login, type LoginCredentials } from '$lib/api/session.js';
	import { ApiError } from '$lib/api/types.js';
	import LoginForm from '$lib/components/login-form.svelte';
	import SessionTransition from '$lib/components/session-transition.svelte';

	const AUTH_TRANSITION_DELAY_MS = 700;

	let isSubmitting = $state(false);
	let errorMessage = $state<string | null>(null);
	let transitionMessage = $state<string | null>(null);
	const redirectTo = $derived(getRedirectTo(page.url.searchParams.get('redirectTo')));

	async function handleLogin(credentials: LoginCredentials) {
		isSubmitting = true;
		errorMessage = null;
		transitionMessage = 'Verifying account...';

		try {
			await login(credentials);
			transitionMessage = 'Opening dashboard...';
			await wait(AUTH_TRANSITION_DELAY_MS);
			await goto(redirectTo, { replaceState: true });
		} catch (error) {
			transitionMessage = null;
			errorMessage = getLoginErrorMessage(error);
		} finally {
			isSubmitting = false;
		}
	}

	function getLoginErrorMessage(error: unknown) {
		if (error instanceof ApiError) {
			if (error.status === 401) return 'Email atau password tidak valid.';
			return error.message;
		}
		if (error instanceof Error) return error.message;
		return 'Login gagal. Coba beberapa saat lagi.';
	}

	function getRedirectTo(value: string | null) {
		if (!value || !value.startsWith('/') || value.startsWith('//') || value.startsWith('/login')) {
			return '/dashboard';
		}

		return value;
	}

	function wait(ms: number) {
		return new Promise((resolve) => setTimeout(resolve, ms));
	}
</script>

<svelte:head>
	<title>Masuk | POS System</title>
</svelte:head>

{#if transitionMessage}
		<SessionTransition
			class="min-h-svh bg-muted p-6 md:p-10"
			title={transitionMessage}
			description="Please wait a moment."
		/>
{:else}
	<div class="flex min-h-svh flex-col items-center justify-center bg-muted p-6 md:p-10">
		<div class="w-full max-w-sm md:max-w-3xl">
			<LoginForm {isSubmitting} {errorMessage} onSubmit={handleLogin} />
		</div>
	</div>
{/if}
