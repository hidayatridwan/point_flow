<script lang="ts">
	import TrashIcon from '@lucide/svelte/icons/trash';
	import { deleteUser, type User } from '$lib/api/users.js';
	import { ApiError } from '$lib/api/types.js';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import * as AlertDialog from '$lib/components/ui/alert-dialog/index.js';
	import { toast } from 'svelte-sonner';

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
			toast.success('User deleted.', {
				description: `${user.name} was removed.`
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
		return 'Failed to delete user.';
	}
</script>

<AlertDialog.Root bind:open>
	<AlertDialog.Content>
		<AlertDialog.Header>
			<AlertDialog.Title>Delete User?</AlertDialog.Title>
			<AlertDialog.Description>
				This action will delete <span class="font-medium text-foreground">{user.name}</span>.
				The user will no longer be able to sign in.
			</AlertDialog.Description>
		</AlertDialog.Header>

		{#if errorMessage}
			<ErrorAlert title="Delete failed." message={errorMessage} />
		{/if}

		<AlertDialog.Footer>
			<AlertDialog.Cancel disabled={isDeleting}>Cancel</AlertDialog.Cancel>
			<AlertDialog.Action variant="destructive" disabled={isDeleting} onclick={handleDelete}>
				<TrashIcon />
				{isDeleting ? 'Deleting...' : 'Delete'}
			</AlertDialog.Action>
		</AlertDialog.Footer>
	</AlertDialog.Content>
</AlertDialog.Root>
