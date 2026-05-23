<script lang="ts">
	import { getUser, type User } from '$lib/api/users.js';
	import { ApiError } from '$lib/api/types.js';
	import DetailField from '$lib/components/detail-field.svelte';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import StatusBadge from '$lib/components/status-badge.svelte';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import { formatDateTime } from '$lib/formatters.js';
	import UserPermissions from './user-permissions.svelte';

	let {
		userId,
		open = $bindable(false)
	}: {
		userId: number;
		open: boolean;
	} = $props();

	let user = $state<User | null>(null);
	let isLoading = $state(false);
	let errorMessage = $state<string | null>(null);
	let latestRequestId = 0;

	$effect(() => {
		if (!open) return;
		void loadUserDetail(userId);
	});

	async function loadUserDetail(id: number) {
		const requestId = ++latestRequestId;
		isLoading = true;
		errorMessage = null;

		try {
			const result = await getUser(id);
			if (requestId !== latestRequestId) return;
			user = result;
		} catch (error) {
			if (requestId !== latestRequestId) return;
			user = null;
			errorMessage = getErrorMessage(error);
		} finally {
			if (requestId === latestRequestId) {
				isLoading = false;
			}
		}
	}

	function getErrorMessage(error: unknown) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return 'Failed to load user details.';
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="sm:max-w-2xl">
		<Dialog.Header>
			<Dialog.Title>User Details</Dialog.Title>
			<Dialog.Description>Latest information from the user record.</Dialog.Description>
		</Dialog.Header>

		{#if isLoading}
			<div class="grid gap-3">
				<Skeleton class="h-5 w-32" />
				<Skeleton class="h-5 w-52" />
				<Skeleton class="h-5 w-40" />
				<Skeleton class="h-5 w-64" />
			</div>
		{:else if errorMessage}
			<ErrorAlert title="Failed to load details." message={errorMessage} />
		{:else if user}
			<div class="grid gap-4">
				<div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
					<DetailField label="Name" value={user.name} />
					<DetailField label="Email" value={user.email} />
					<DetailField label="Role" value={user.role_name} />
					<DetailField label="Status">
						<StatusBadge active={user.is_active} />
					</DetailField>
					<DetailField label="User ID" value={user.id} />
					<DetailField label="Role Code" value={user.role_code} />
					<DetailField label="Last Login" value={formatDateTime(user.last_login_at)} />
					<DetailField label="Created At" value={formatDateTime(user.created_at)} />
					<DetailField label="Updated At" value={formatDateTime(user.updated_at)} />
				</div>
				<DetailField label="Permissions">
					<UserPermissions permissions={user.permissions} limit={100} />
				</DetailField>
			</div>
		{/if}
	</Dialog.Content>
</Dialog.Root>
