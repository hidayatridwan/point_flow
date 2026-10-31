<script lang="ts">
	import CheckCircleIcon from '@lucide/svelte/icons/circle-check';
	import { postStockConversion, type StockConversion } from '$lib/api/stock-conversions.js';
	import { ApiError } from '$lib/api/types.js';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import * as AlertDialog from '$lib/components/ui/alert-dialog/index.js';
	import { Spinner } from '$lib/components/ui/spinner/index.js';
	import { toast } from 'svelte-sonner';
	import { t } from '$lib/i18n/locale.svelte.js';

	let {
		stockConversion,
		open = $bindable(false),
		onPosted
	}: {
		stockConversion: StockConversion;
		open: boolean;
		onPosted?: (stockConversion: StockConversion) => void | Promise<void>;
	} = $props();

	let isSubmitting = $state(false);
	let errorMessage = $state<string | null>(null);

	$effect(() => {
		if (!open) return;
		errorMessage = null;
	});

	async function handlePost(event: MouseEvent) {
		event.preventDefault();
		if (isSubmitting) return;

		isSubmitting = true;
		errorMessage = null;

		try {
			const postedStockConversion = await postStockConversion(stockConversion.id);
			await onPosted?.(postedStockConversion);
			toast.success(t('inventory.stockConversions.post.postedToast'), {
				description: `${postedStockConversion.conversion_no} updated product SKU stock successfully.`
			});
			open = false;
		} catch (error) {
			errorMessage = getErrorMessage(error);
		} finally {
			isSubmitting = false;
		}
	}

	function getErrorMessage(error: unknown) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return t('inventory.stockConversions.post.postFailed');
	}
</script>

<AlertDialog.Root bind:open>
	<AlertDialog.Content>
		<AlertDialog.Header>
			<AlertDialog.Title>{t('inventory.stockConversions.post.title')}</AlertDialog.Title>
			<AlertDialog.Description>
				{t('inventory.stockConversions.post.desc').replace(
					'{convNo}',
					stockConversion.conversion_no
				)}
			</AlertDialog.Description>
		</AlertDialog.Header>

		{#if errorMessage}
			<ErrorAlert title={t('common.postFailed')} message={errorMessage} />
		{/if}

		<AlertDialog.Footer>
			<AlertDialog.Cancel disabled={isSubmitting}
				>{t('inventory.stockConversions.post.cancelBtn')}</AlertDialog.Cancel
			>
			<AlertDialog.Action disabled={isSubmitting} onclick={handlePost}>
				{#if isSubmitting}
					<Spinner />
				{:else}
					<CheckCircleIcon />
				{/if}
				{isSubmitting
					? t('inventory.stockConversions.post.posting')
					: t('inventory.stockConversions.post.postBtn')}
			</AlertDialog.Action>
		</AlertDialog.Footer>
	</AlertDialog.Content>
</AlertDialog.Root>
