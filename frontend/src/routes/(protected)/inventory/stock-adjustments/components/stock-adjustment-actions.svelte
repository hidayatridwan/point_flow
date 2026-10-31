<script lang="ts">
	import BanIcon from '@lucide/svelte/icons/ban';
	import CheckCircleIcon from '@lucide/svelte/icons/circle-check';
	import EllipsisIcon from '@lucide/svelte/icons/ellipsis';
	import EyeIcon from '@lucide/svelte/icons/eye';
	import type { StockAdjustment } from '$lib/api/stock-adjustments.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js';
	import { t } from '$lib/i18n/locale.svelte.js';
	import StockAdjustmentCancelDialog from './stock-adjustment-cancel-dialog.svelte';
	import StockAdjustmentDetailDialog from './stock-adjustment-detail-dialog.svelte';
	import StockAdjustmentPostAlertDialog from './stock-adjustment-post-alert-dialog.svelte';

	let {
		stockAdjustment,
		onChanged
	}: {
		stockAdjustment: StockAdjustment;
		onChanged?: (stockAdjustment: StockAdjustment) => void | Promise<void>;
	} = $props();

	let isDetailOpen = $state(false);
	let isPostOpen = $state(false);
	let isCancelOpen = $state(false);

	const canPost = $derived(stockAdjustment.status === 'draft');
	const canCancel = $derived(stockAdjustment.status === 'draft');

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
					aria-label={t('common.openActions').replace('{docNo}', stockAdjustment.adjustment_no)}
					title={t('common.action')}
				>
					<EllipsisIcon />
				</Button>
			{/snippet}
		</DropdownMenu.Trigger>
		<DropdownMenu.Content align="end" class="min-w-48">
			<DropdownMenu.Item onSelect={openDetail}>
				<EyeIcon />
				{t('common.detail')}
			</DropdownMenu.Item>
			{#if canPost}
				<DropdownMenu.Item onSelect={openPost}>
					<CheckCircleIcon />
					{t('inventory.stockAdjustments.post.postBtn')}
				</DropdownMenu.Item>
			{/if}
			{#if canCancel}
				<DropdownMenu.Item variant="destructive" onSelect={openCancel}>
					<BanIcon />
					{t('inventory.stockAdjustments.cancel.title')}
				</DropdownMenu.Item>
			{/if}
		</DropdownMenu.Content>
	</DropdownMenu.Root>

	<StockAdjustmentDetailDialog stockAdjustmentId={stockAdjustment.id} bind:open={isDetailOpen} />
	<StockAdjustmentPostAlertDialog {stockAdjustment} bind:open={isPostOpen} onPosted={onChanged} />
	<StockAdjustmentCancelDialog {stockAdjustment} bind:open={isCancelOpen} onCancelled={onChanged} />
</div>
