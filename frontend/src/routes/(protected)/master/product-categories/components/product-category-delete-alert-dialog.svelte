<script lang="ts">
	import CircleAlertIcon from '@lucide/svelte/icons/circle-alert';
	import TrashIcon from '@lucide/svelte/icons/trash';
	import { deleteProductCategory, type ProductCategory } from '$lib/api/product-categories.js';
	import { ApiError } from '$lib/api/types.js';
	import * as Alert from '$lib/components/ui/alert/index.js';
	import * as AlertDialog from '$lib/components/ui/alert-dialog/index.js';
	import { toast } from 'svelte-sonner';

	let {
		category,
		open = $bindable(false),
		onDeleted
	}: {
		category: ProductCategory;
		open: boolean;
		onDeleted?: (categoryId: number) => void | Promise<void>;
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
			await deleteProductCategory(category.id);
			await onDeleted?.(category.id);
			toast.success('Product category deleted.', {
				description: `${category.name} was removed.`
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
		return 'Failed to delete product category.';
	}
</script>

<AlertDialog.Root bind:open>
	<AlertDialog.Content>
		<AlertDialog.Header>
			<AlertDialog.Title>Delete Product Category?</AlertDialog.Title>
			<AlertDialog.Description>
				This action will delete <span class="font-medium text-foreground">{category.name}</span>.
				This category will no longer appear in the product category list.
			</AlertDialog.Description>
		</AlertDialog.Header>

		{#if errorMessage}
			<Alert.Root variant="destructive">
				<CircleAlertIcon />
				<Alert.Title>Delete failed.</Alert.Title>
				<Alert.Description>{errorMessage}</Alert.Description>
			</Alert.Root>
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
