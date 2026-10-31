<script lang="ts">
	import TrashIcon from '@lucide/svelte/icons/trash';
	import { deleteRole, type Role } from '$lib/api/roles.js';
	import { ApiError } from '$lib/api/types.js';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import * as AlertDialog from '$lib/components/ui/alert-dialog/index.js';
	import { toast } from 'svelte-sonner';
	import { t } from '$lib/i18n/locale.svelte.js';

	let {
		role,
		open = $bindable(false),
		onDeleted
	}: {
		role: Role;
		open: boolean;
		onDeleted?: (roleId: number) => void | Promise<void>;
	} = $props();

	let isDeleting = $state(false);
	let errorMessage = $state<string | null>(null);

	$effect(() => {
		if (!open) return;
		errorMessage = null;
	});

	async function handleDelete(event: MouseEvent) {
		event.preventDefault();
		if (isDeleting || role.is_system) return;

		isDeleting = true;
		errorMessage = null;

		try {
			await deleteRole(role.id);
			await onDeleted?.(role.id);
			toast.success(t('config.roles.deletedToast'), {
				description: `${role.name} ${t('common.wasRemoved')}`
			});
			open = false;
		} catch (error) {
			errorMessage = getErrorMessage(error);
		} finally {
			isDeleting = false;
		}
	}

	function getErrorMessage(error: unknown) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return t('config.roles.errorDelete');
	}
</script>

<AlertDialog.Root bind:open>
	<AlertDialog.Content>
		<AlertDialog.Header>
			<AlertDialog.Title>{t('config.roles.deleteTitle')}</AlertDialog.Title>
			<AlertDialog.Description>
				{t('config.roles.deleteDesc').replace('{name}', role.name)}
				{t('config.roles.deleteDescExtra')}
			</AlertDialog.Description>
		</AlertDialog.Header>

		{#if errorMessage}
			<ErrorAlert title={t('common.deleteFailed')} message={errorMessage} />
		{/if}

		<AlertDialog.Footer>
			<AlertDialog.Cancel disabled={isDeleting}>{t('common.cancel')}</AlertDialog.Cancel>
			<AlertDialog.Action
				variant="destructive"
				disabled={isDeleting || role.is_system}
				onclick={handleDelete}
			>
				<TrashIcon />
				{isDeleting ? t('common.deleting') : t('common.delete')}
			</AlertDialog.Action>
		</AlertDialog.Footer>
	</AlertDialog.Content>
</AlertDialog.Root>
