<script lang="ts">
	import CheckCircleIcon from '@lucide/svelte/icons/circle-check';
	import { postGoodsReceipt, type GoodsReceipt } from '$lib/api/goods-receipts.js';
	import { ApiError } from '$lib/api/types.js';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import * as AlertDialog from '$lib/components/ui/alert-dialog/index.js';
	import { Spinner } from '$lib/components/ui/spinner/index.js';
	import { toast } from 'svelte-sonner';

	let {
		goodsReceipt,
		open = $bindable(false),
		onPosted
	}: {
		goodsReceipt: GoodsReceipt;
		open: boolean;
		onPosted?: (goodsReceipt: GoodsReceipt) => void | Promise<void>;
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
			const postedGoodsReceipt = await postGoodsReceipt(goodsReceipt.id);
			await onPosted?.(postedGoodsReceipt);
			toast.success('Goods receipt posted.', {
				description: `${postedGoodsReceipt.gr_no} updated stock successfully.`
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
		return 'Failed to post goods receipt.';
	}
</script>

<AlertDialog.Root bind:open>
	<AlertDialog.Content>
		<AlertDialog.Header>
			<AlertDialog.Title>Post Goods Receipt?</AlertDialog.Title>
			<AlertDialog.Description>
				This action will post <span class="font-medium text-foreground">{goodsReceipt.gr_no}</span>
				and increase product SKU stock from the received quantities.
			</AlertDialog.Description>
		</AlertDialog.Header>

		{#if errorMessage}
			<ErrorAlert title="Post failed." message={errorMessage} />
		{/if}

		<AlertDialog.Footer>
			<AlertDialog.Cancel disabled={isSubmitting}>Cancel</AlertDialog.Cancel>
			<AlertDialog.Action disabled={isSubmitting} onclick={handlePost}>
				{#if isSubmitting}
					<Spinner />
				{:else}
					<CheckCircleIcon />
				{/if}
				{isSubmitting ? 'Posting...' : 'Post receipt'}
			</AlertDialog.Action>
		</AlertDialog.Footer>
	</AlertDialog.Content>
</AlertDialog.Root>
