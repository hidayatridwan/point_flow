<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { onMount } from 'svelte';
	import CircleUserIcon from '@lucide/svelte/icons/circle-user';
	import KeyRoundIcon from '@lucide/svelte/icons/key-round';
	import RefreshCwIcon from '@lucide/svelte/icons/refresh-cw';
	import { clearAccessToken } from '$lib/api/auth.svelte.js';
	import { changePassword, getCurrentUser, type AuthUser } from '$lib/api/session.js';
	import { ApiError } from '$lib/api/types.js';
	import DetailField from '$lib/components/detail-field.svelte';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import PageHeader from '$lib/components/page-header.svelte';
	import { Badge } from '$lib/components/ui/badge/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import { formatDateTime } from '$lib/formatters.js';
	import { toast } from 'svelte-sonner';

	let currentUser = $state<AuthUser | null>(null);
	let currentPassword = $state('');
	let newPassword = $state('');
	let confirmNewPassword = $state('');
	let isLoading = $state(true);
	let isChangingPassword = $state(false);
	let errorMessage = $state<string | null>(null);
	let passwordErrorMessage = $state<string | null>(null);

	onMount(() => {
		void loadCurrentUser();
	});

	async function loadCurrentUser() {
		isLoading = true;
		errorMessage = null;

		try {
			currentUser = await getCurrentUser();
		} catch (error) {
			currentUser = null;
			errorMessage = getErrorMessage(error);
		} finally {
			isLoading = false;
		}
	}

	async function handleChangePassword(event: SubmitEvent) {
		event.preventDefault();
		if (isChangingPassword) return;

		const validationError = getChangePasswordValidationError();
		if (validationError) {
			passwordErrorMessage = validationError;
			return;
		}

		isChangingPassword = true;
		passwordErrorMessage = null;

		try {
			await changePassword({
				current_password: currentPassword,
				new_password: newPassword
			});
			clearAccessToken();
			toast.success('Password changed.', {
				description: 'Please sign in again with your new password.'
			});
			await goto(resolve('/'), { invalidateAll: true, replaceState: true });
		} catch (error) {
			passwordErrorMessage = getChangePasswordErrorMessage(error);
		} finally {
			isChangingPassword = false;
		}
	}

	function getChangePasswordValidationError() {
		if (!currentPassword.trim()) return 'Current password is required.';
		if (!newPassword) return 'New password is required.';
		if (newPassword.length < 8) return 'New password must contain at least 8 characters.';
		if (newPassword !== confirmNewPassword) return 'Confirm password must match new password.';
		return null;
	}

	function formatRole(role: string) {
		return role
			.split('_')
			.map((segment) => segment.charAt(0).toUpperCase() + segment.slice(1))
			.join(' ');
	}

	function getInitials(name: string) {
		return name
			.split(/\s+/)
			.filter(Boolean)
			.slice(0, 2)
			.map((segment) => segment.charAt(0).toUpperCase())
			.join('');
	}

	function getErrorMessage(error: unknown) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return 'Failed to load current user.';
	}

	function getChangePasswordErrorMessage(error: unknown) {
		if (error instanceof ApiError) {
			if (error.status === 401) return 'Current password is incorrect.';
			return error.message;
		}
		if (error instanceof Error) return error.message;
		return 'Failed to change password.';
	}
</script>

<svelte:head>
	<title>Current User | POS System</title>
</svelte:head>

<div class="flex flex-col gap-6">
	<PageHeader
		title="Current User"
		description="View the account currently signed in to this session."
	>
		{#snippet actions()}
			<Button type="button" variant="outline" class="w-full sm:w-auto" onclick={loadCurrentUser}>
				<RefreshCwIcon />
				Refresh
			</Button>
		{/snippet}
	</PageHeader>

	{#if isLoading}
		<div class="grid gap-4 lg:grid-cols-[320px_minmax(0,1fr)]">
			<div class="rounded-md border p-5">
				<Skeleton class="mx-auto size-20 rounded-full" />
				<Skeleton class="mx-auto mt-4 h-5 w-40" />
				<Skeleton class="mx-auto mt-2 h-4 w-56" />
			</div>
			<div class="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
				{#each Array.from({ length: 6 }).keys() as index (index)}
					<Skeleton class="h-20 rounded-md" />
				{/each}
			</div>
		</div>
	{:else if errorMessage}
		<div class="grid gap-4">
			<ErrorAlert title="Failed to load current user." message={errorMessage} />
			<div>
				<Button type="button" variant="outline" onclick={loadCurrentUser}>
					<RefreshCwIcon />
					Try again
				</Button>
			</div>
		</div>
	{:else if currentUser}
		<div class="grid gap-4 lg:grid-cols-[320px_minmax(0,1fr)]">
			<section
				class="flex flex-col items-center gap-4 rounded-md border bg-muted/20 p-6 text-center"
			>
				<div
					class="flex size-20 items-center justify-center rounded-full bg-primary text-xl font-semibold text-primary-foreground"
				>
					{#if getInitials(currentUser.name)}
						{getInitials(currentUser.name)}
					{:else}
						<CircleUserIcon class="size-8" />
					{/if}
				</div>
				<div class="min-w-0">
					<h2 class="truncate text-lg font-semibold">{currentUser.name}</h2>
					<p class="truncate text-sm text-muted-foreground">{currentUser.email}</p>
				</div>
				<div class="flex flex-wrap justify-center gap-2">
					<Badge variant="outline">{formatRole(currentUser.role)}</Badge>
					<Badge
						variant="outline"
						class={currentUser.is_active
							? 'border-emerald-600/20 bg-emerald-50 text-emerald-700 dark:bg-emerald-950/30 dark:text-emerald-300'
							: 'border-rose-600/20 bg-rose-50 text-rose-700 dark:bg-rose-950/30 dark:text-rose-300'}
					>
						{currentUser.is_active ? 'Active' : 'Inactive'}
					</Badge>
				</div>
			</section>

			<section class="grid gap-3 sm:grid-cols-2 xl:grid-cols-3" aria-label="Current user details">
				<DetailField label="User ID" value={currentUser.id} />
				<DetailField label="Name" value={currentUser.name} />
				<DetailField label="Email" value={currentUser.email} />
				<DetailField label="Role" value={formatRole(currentUser.role)} />
				<DetailField label="Status" value={currentUser.is_active ? 'Active' : 'Inactive'} />
				<DetailField label="Last Login" value={formatDateTime(currentUser.last_login_at)} />
				<DetailField label="Created At" value={formatDateTime(currentUser.created_at)} />
				<DetailField label="Updated At" value={formatDateTime(currentUser.updated_at)} />
				<DetailField label="Deleted At" value={formatDateTime(currentUser.deleted_at)} />
			</section>

			<section
				class="rounded-md border bg-background p-5 lg:col-span-2"
				aria-label="Change password"
			>
				<form class="grid gap-5" onsubmit={handleChangePassword}>
					<div class="flex items-center gap-3">
						<div
							class="flex size-10 shrink-0 items-center justify-center rounded-md bg-muted text-muted-foreground"
						>
							<KeyRoundIcon class="size-5" />
						</div>
						<div class="min-w-0">
							<h2 class="text-base font-semibold">Change Password</h2>
							<p class="text-sm text-muted-foreground">
								Update the password used to sign in to this account.
							</p>
						</div>
					</div>

					{#if passwordErrorMessage}
						<ErrorAlert title="Failed to change password." message={passwordErrorMessage} />
					{/if}

					<div class="grid gap-4 md:grid-cols-3">
						<div class="grid gap-2">
							<Label for="current-password">Current Password</Label>
							<Input
								id="current-password"
								type="password"
								autocomplete="current-password"
								disabled={isChangingPassword}
								bind:value={currentPassword}
							/>
						</div>
						<div class="grid gap-2">
							<Label for="new-password">New Password</Label>
							<Input
								id="new-password"
								type="password"
								autocomplete="new-password"
								disabled={isChangingPassword}
								bind:value={newPassword}
							/>
						</div>
						<div class="grid gap-2">
							<Label for="confirm-new-password">Confirm New Password</Label>
							<Input
								id="confirm-new-password"
								type="password"
								autocomplete="new-password"
								disabled={isChangingPassword}
								bind:value={confirmNewPassword}
							/>
						</div>
					</div>

					<div class="flex justify-end">
						<Button type="submit" class="w-full sm:w-auto" disabled={isChangingPassword}>
							<KeyRoundIcon />
							{isChangingPassword ? 'Changing...' : 'Change Password'}
						</Button>
					</div>
				</form>
			</section>
		</div>
	{/if}
</div>
