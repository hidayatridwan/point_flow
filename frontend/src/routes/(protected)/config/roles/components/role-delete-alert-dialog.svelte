<script lang="ts">
	import TrashIcon from '@lucide/svelte/icons/trash';
	import { deleteRole, type Role } from '$lib/api/roles.js';
	import { ApiError } from '$lib/api/types.js';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import * as AlertDialog from '$lib/components/ui/alert-dialog/index.js';
	import { toast } from 'svelte-sonner';

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
			toast.success('Role deleted.', {
				description: `${role.name} was removed.`
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
		return 'Failed to delete role.';
	}
</script>

<AlertDialog.Root bind:open>
	<AlertDialog.Content>
		<AlertDialog.Header>
			<AlertDialog.Title>Delete Role?</AlertDialog.Title>
			<AlertDialog.Description>
				This action will delete <span class="font-medium text-foreground">{role.name}</span>. This
				role will no longer appear in role selection lists.
			</AlertDialog.Description>
		</AlertDialog.Header>

		{#if errorMessage}
			<ErrorAlert title="Delete failed." message={errorMessage} />
		{/if}

		<AlertDialog.Footer>
			<AlertDialog.Cancel disabled={isDeleting}>Cancel</AlertDialog.Cancel>
			<AlertDialog.Action
				variant="destructive"
				disabled={isDeleting || role.is_system}
				onclick={handleDelete}
			>
				<TrashIcon />
				{isDeleting ? 'Deleting...' : 'Delete'}
			</AlertDialog.Action>
		</AlertDialog.Footer>
	</AlertDialog.Content>
</AlertDialog.Root>
