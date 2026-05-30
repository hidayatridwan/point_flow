<script lang="ts">
	import CircleAlertIcon from '@lucide/svelte/icons/circle-alert';
	import TrashIcon from '@lucide/svelte/icons/trash';
	import { deleteProductSku, type ProductSku } from '$lib/api/product-skus.js';
	import { ApiError } from '$lib/api/types.js';
	import * as Alert from '$lib/components/ui/alert/index.js';
	import * as AlertDialog from '$lib/components/ui/alert-dialog/index.js';
	import { toast } from 'svelte-sonner';
	import { t } from '$lib/i18n/locale.svelte.js';

	let {
		productSku,
		open = $bindable(false),
		onDeleted
	}: {
		productSku: ProductSku;
		open: boolean;
		onDeleted?: (productSkuId: number) => void | Promise<void>;
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
			await deleteProductSku(productSku.id);
			await onDeleted?.(productSku.id);
			toast.success(t('master.productSkus.deletedToast'), {
				description: `${productSku.sku} was removed.`
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
		return t('master.productSkus.errorDelete');
	}
</script>

<AlertDialog.Root bind:open>
	<AlertDialog.Content>
		<AlertDialog.Header>
			<AlertDialog.Title>Delete Product SKU?</AlertDialog.Title>
			<AlertDialog.Description>
				This action will delete SKU
				<span class="font-medium text-foreground">{productSku.sku}</span>
				for <span class="font-medium text-foreground">{productSku.product_name}</span>.
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
			<AlertDialog.Cancel disabled={isDeleting}>{t('common.cancel')}</AlertDialog.Cancel>
			<AlertDialog.Action variant="destructive" disabled={isDeleting} onclick={handleDelete}>
				<TrashIcon />
				{isDeleting ? t('common.deleting') : t('common.delete')}
			</AlertDialog.Action>
		</AlertDialog.Footer>
	</AlertDialog.Content>
</AlertDialog.Root>
