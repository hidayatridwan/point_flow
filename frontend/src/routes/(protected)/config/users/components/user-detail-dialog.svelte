<script lang="ts">
	import { getUser, type User } from '$lib/api/users.js';
	import { ApiError } from '$lib/api/types.js';
	import DetailField from '$lib/components/detail-field.svelte';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import StatusBadge from '$lib/components/status-badge.svelte';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import { formatDateTime } from '$lib/formatters.js';
	import { t } from '$lib/i18n/locale.svelte.js';
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
		return t('config.users.loadFailed');
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="sm:max-w-2xl">
		<Dialog.Header>
			<Dialog.Title>{t('config.users.detailTitle')}</Dialog.Title>
			<Dialog.Description>{t('config.users.detailDesc')}</Dialog.Description>
		</Dialog.Header>

		{#if isLoading}
			<div class="grid gap-3">
				<Skeleton class="h-5 w-32" />
				<Skeleton class="h-5 w-52" />
				<Skeleton class="h-5 w-40" />
				<Skeleton class="h-5 w-64" />
			</div>
		{:else if errorMessage}
			<ErrorAlert title={t('common.loadFailed')} message={errorMessage} />
		{:else if user}
			<div class="grid gap-4">
				<div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
					<DetailField label={t('common.name')} value={user.name} />
					<DetailField label={t('common.email')} value={user.email} />
					<DetailField label={t('config.users.role')} value={user.role_name} />
					<DetailField label={t('common.status')}>
						<StatusBadge active={user.is_active} />
					</DetailField>
					<DetailField label={t('config.users.userId')} value={user.id} />
					<DetailField label={t('config.users.roleCode')} value={user.role_code} />
					<DetailField
						label={t('config.users.lastLogin')}
						value={formatDateTime(user.last_login_at)}
					/>
					<DetailField label={t('common.createdAt')} value={formatDateTime(user.created_at)} />
					<DetailField label={t('common.updatedAt')} value={formatDateTime(user.updated_at)} />
				</div>
				<DetailField label={t('config.users.colPermissions')}>
					<UserPermissions permissions={user.permissions} limit={100} />
				</DetailField>
			</div>
		{/if}
	</Dialog.Content>
</Dialog.Root>
