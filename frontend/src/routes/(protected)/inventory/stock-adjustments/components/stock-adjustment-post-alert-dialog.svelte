<script lang="ts">
	import CheckCircleIcon from '@lucide/svelte/icons/circle-check';
	import { postStockAdjustment, type StockAdjustment } from '$lib/api/stock-adjustments.js';
	import { ApiError } from '$lib/api/types.js';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import * as AlertDialog from '$lib/components/ui/alert-dialog/index.js';
	import { Spinner } from '$lib/components/ui/spinner/index.js';
	import { toast } from 'svelte-sonner';
	import { t } from '$lib/i18n/locale.svelte.js';

	let {
		stockAdjustment,
		open = $bindable(false),
		onPosted
	}: {
		stockAdjustment: StockAdjustment;
		open: boolean;
		onPosted?: (stockAdjustment: StockAdjustment) => void | Promise<void>;
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
			const postedStockAdjustment = await postStockAdjustment(stockAdjustment.id);
			await onPosted?.(postedStockAdjustment);
			toast.success(t('inventory.stockAdjustments.post.postedToast'), {
				description: `${postedStockAdjustment.adjustment_no} updated stock successfully.`
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
		return t('inventory.stockAdjustments.post.postFailed');
	}
</script>

<AlertDialog.Root bind:open>
	<AlertDialog.Content>
		<AlertDialog.Header>
			<AlertDialog.Title>Post Stock Adjustment?</AlertDialog.Title>
			<AlertDialog.Description>
				This action will post
				<span class="font-medium text-foreground">{stockAdjustment.adjustment_no}</span>
				and update product SKU stock to the adjusted quantities.
			</AlertDialog.Description>
		</AlertDialog.Header>

		{#if errorMessage}
			<ErrorAlert title={t('common.postFailed')} message={errorMessage} />
		{/if}

		<AlertDialog.Footer>
			<AlertDialog.Cancel disabled={isSubmitting}>Cancel</AlertDialog.Cancel>
			<AlertDialog.Action disabled={isSubmitting} onclick={handlePost}>
				{#if isSubmitting}
					<Spinner />
				{:else}
					<CheckCircleIcon />
				{/if}
				{isSubmitting
					? t('inventory.stockAdjustments.post.posting')
					: t('inventory.stockAdjustments.post.postBtn')}
			</AlertDialog.Action>
		</AlertDialog.Footer>
	</AlertDialog.Content>
</AlertDialog.Root>
