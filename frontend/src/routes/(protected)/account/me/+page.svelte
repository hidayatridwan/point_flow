<script lang="ts">
	import { onMount } from 'svelte';
	import CircleUserIcon from '@lucide/svelte/icons/circle-user';
	import RefreshCwIcon from '@lucide/svelte/icons/refresh-cw';
	import { getCurrentUser, type AuthUser } from '$lib/api/session.js';
	import { ApiError } from '$lib/api/types.js';
	import DetailField from '$lib/components/detail-field.svelte';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import PageHeader from '$lib/components/page-header.svelte';
	import { Badge } from '$lib/components/ui/badge/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import { formatDateTime } from '$lib/formatters.js';

	let currentUser = $state<AuthUser | null>(null);
	let isLoading = $state(true);
	let errorMessage = $state<string | null>(null);

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
		</div>
	{/if}
</div>
