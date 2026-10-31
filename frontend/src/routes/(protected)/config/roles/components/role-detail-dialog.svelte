<script lang="ts">
	import { getRole, type Role } from '$lib/api/roles.js';
	import { ApiError } from '$lib/api/types.js';
	import DetailField from '$lib/components/detail-field.svelte';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import StatusBadge from '$lib/components/status-badge.svelte';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import { formatDateTime } from '$lib/formatters.js';
	import { t } from '$lib/i18n/locale.svelte.js';
	import RolePermissions from './role-permissions.svelte';

	let {
		roleId,
		open = $bindable(false)
	}: {
		roleId: number;
		open: boolean;
	} = $props();

	let role = $state<Role | null>(null);
	let isLoading = $state(false);
	let errorMessage = $state<string | null>(null);
	let latestRequestId = 0;

	$effect(() => {
		if (!open) return;
		void loadRoleDetail(roleId);
	});

	async function loadRoleDetail(id: number) {
		const requestId = ++latestRequestId;
		isLoading = true;
		errorMessage = null;

		try {
			const result = await getRole(id);
			if (requestId !== latestRequestId) return;
			role = result;
		} catch (error) {
			if (requestId !== latestRequestId) return;
			role = null;
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
		return t('config.roles.loadFailed');
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="sm:max-w-2xl">
		<Dialog.Header>
			<Dialog.Title>{t('config.roles.detailTitle')}</Dialog.Title>
			<Dialog.Description>{t('config.roles.detailDesc')}</Dialog.Description>
		</Dialog.Header>

		{#if isLoading}
			<div class="grid gap-3">
				<Skeleton class="h-5 w-32" />
				<Skeleton class="h-5 w-48" />
				<Skeleton class="h-5 w-64" />
				<Skeleton class="h-5 w-40" />
			</div>
		{:else if errorMessage}
			<ErrorAlert title={t('common.loadFailed')} message={errorMessage} />
		{:else if role}
			<div class="grid gap-4">
				<div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
					<DetailField label={t('config.roles.code')} value={role.code} />
					<DetailField label={t('common.name')} value={role.name} />
					<DetailField label={t('config.roles.type')}>
						<StatusBadge
							active={role.is_system}
							activeLabel={t('config.roles.typeSystem')}
							inactiveLabel={t('config.roles.typeCustom')}
						/>
					</DetailField>
					<DetailField label={t('common.description')} value={role.description ?? '-'} />
					<DetailField label={t('common.id')} value={role.id} />
					<DetailField label={t('common.createdAt')} value={formatDateTime(role.created_at)} />
					<DetailField label={t('common.updatedAt')} value={formatDateTime(role.updated_at)} />
				</div>
				<DetailField label={t('config.roles.permissions')}>
					<RolePermissions permissions={role.permissions} limit={100} />
				</DetailField>
			</div>
		{/if}
	</Dialog.Content>
</Dialog.Root>
