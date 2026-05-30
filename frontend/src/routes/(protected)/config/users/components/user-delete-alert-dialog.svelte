<script lang="ts">
	import TrashIcon from '@lucide/svelte/icons/trash';
	import { deleteUser, type User } from '$lib/api/users.js';
	import { ApiError } from '$lib/api/types.js';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import * as AlertDialog from '$lib/components/ui/alert-dialog/index.js';
	import { toast } from 'svelte-sonner';
	import { t } from '$lib/i18n/locale.svelte.js';

	let {
		user,
		open = $bindable(false),
		onDeleted
	}: {
		user: User;
		open: boolean;
		onDeleted?: (userId: number) => void | Promise<void>;
	} = $props();

	let isDeleting = $state(false);
	let errorMessage = $state<string | null>(null);

	$effect(() => {
		if (!open) return;
		errorMessage = null;
	});

	async function handleDelete(event: MouseEvent) {
		event.preventDefault();
		if (isDeleting) return;

		isDeleting = true;
		errorMessage = null;

		try {
			await deleteUser(user.id);
			await onDeleted?.(user.id);
			toast.success(t('config.users.deletedToast'), {
				description: `${user.name} ${t('common.wasRemoved')}`
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
		return t('config.users.errorDelete');
	}
</script>

<AlertDialog.Root bind:open>
	<AlertDialog.Content>
		<AlertDialog.Header>
			<AlertDialog.Title>{t('config.users.deleteTitle')}</AlertDialog.Title>
			<AlertDialog.Description>
				<span class="font-medium text-foreground">{user.name}</span> — {t('config.users.deleteDesc')}
			</AlertDialog.Description>
		</AlertDialog.Header>

		{#if errorMessage}
			<ErrorAlert title={t('common.deleteFailed')} message={errorMessage} />
		{/if}

		<AlertDialog.Footer>
			<AlertDialog.Cancel disabled={isDeleting}>{t('common.cancel')}</AlertDialog.Cancel>
			<AlertDialog.Action variant="destructive" disabled={isDeleting} onclick={handleDelete}>
				<TrashIcon />
				{isDeleting ? t('common.deleting') : t('common.delete')}
			</AlertDialog.Action>
		</AlertDialog.Footer>
	</AlertDialog.Content>
</AlertDialog.Root>
