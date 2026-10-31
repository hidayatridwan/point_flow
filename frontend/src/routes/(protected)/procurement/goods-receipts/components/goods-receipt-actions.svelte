<script lang="ts">
	import BanIcon from '@lucide/svelte/icons/ban';
	import CheckCircleIcon from '@lucide/svelte/icons/circle-check';
	import EllipsisIcon from '@lucide/svelte/icons/ellipsis';
	import EyeIcon from '@lucide/svelte/icons/eye';
	import type { GoodsReceipt } from '$lib/api/goods-receipts.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js';
	import { t } from '$lib/i18n/locale.svelte.js';
	import GoodsReceiptCancelDialog from './goods-receipt-cancel-dialog.svelte';
	import GoodsReceiptDetailDialog from './goods-receipt-detail-dialog.svelte';
	import GoodsReceiptPostAlertDialog from './goods-receipt-post-alert-dialog.svelte';

	let {
		goodsReceipt,
		onChanged
	}: {
		goodsReceipt: GoodsReceipt;
		onChanged?: (goodsReceipt: GoodsReceipt) => void | Promise<void>;
	} = $props();

	let isDetailOpen = $state(false);
	let isPostOpen = $state(false);
	let isCancelOpen = $state(false);

	const canPost = $derived(goodsReceipt.status === 'draft');
	const canCancel = $derived(goodsReceipt.status === 'draft');

	function openDetail() {
		isDetailOpen = true;
	}

	function openPost() {
		isPostOpen = true;
	}

	function openCancel() {
		isCancelOpen = true;
	}
</script>

<div class="flex justify-center">
	<DropdownMenu.Root>
		<DropdownMenu.Trigger>
			{#snippet child({ props })}
				<Button
					{...props}
					variant="ghost"
					size="icon-sm"
					class="size-8"
					aria-label={t('common.openActions').replace('{docNo}', goodsReceipt.gr_no)}
					title={t('common.action')}
				>
					<EllipsisIcon />
				</Button>
			{/snippet}
		</DropdownMenu.Trigger>
		<DropdownMenu.Content align="end" class="min-w-44">
			<DropdownMenu.Item onSelect={openDetail}>
				<EyeIcon />
				{t('common.detail')}
			</DropdownMenu.Item>
			{#if canPost}
				<DropdownMenu.Item onSelect={openPost}>
					<CheckCircleIcon />
					{t('procurement.goodsReceipts.post.postBtn')}
				</DropdownMenu.Item>
			{/if}
			{#if canCancel}
				<DropdownMenu.Item variant="destructive" onSelect={openCancel}>
					<BanIcon />
					{t('procurement.goodsReceipts.cancel.title')}
				</DropdownMenu.Item>
			{/if}
		</DropdownMenu.Content>
	</DropdownMenu.Root>

	<GoodsReceiptDetailDialog goodsReceiptId={goodsReceipt.id} bind:open={isDetailOpen} />
	<GoodsReceiptPostAlertDialog {goodsReceipt} bind:open={isPostOpen} onPosted={onChanged} />
	<GoodsReceiptCancelDialog {goodsReceipt} bind:open={isCancelOpen} onCancelled={onChanged} />
</div>
