<script lang="ts">
	import BanIcon from '@lucide/svelte/icons/ban';
	import CheckCircleIcon from '@lucide/svelte/icons/circle-check';
	import EllipsisIcon from '@lucide/svelte/icons/ellipsis';
	import EyeIcon from '@lucide/svelte/icons/eye';
	import type { StockConversion } from '$lib/api/stock-conversions.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js';
	import StockConversionCancelDialog from './stock-conversion-cancel-dialog.svelte';
	import StockConversionDetailDialog from './stock-conversion-detail-dialog.svelte';
	import StockConversionPostAlertDialog from './stock-conversion-post-alert-dialog.svelte';

	let {
		stockConversion,
		onChanged
	}: {
		stockConversion: StockConversion;
		onChanged?: (stockConversion: StockConversion) => void | Promise<void>;
	} = $props();

	let isDetailOpen = $state(false);
	let isPostOpen = $state(false);
	let isCancelOpen = $state(false);

	const canPost = $derived(stockConversion.status === 'draft');
	const canCancel = $derived(stockConversion.status === 'draft');

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
					aria-label={`Open actions for ${stockConversion.conversion_no}`}
					title="Actions"
				>
					<EllipsisIcon />
				</Button>
			{/snippet}
		</DropdownMenu.Trigger>
		<DropdownMenu.Content align="end" class="min-w-48">
			<DropdownMenu.Item onSelect={openDetail}>
				<EyeIcon />
				Detail
			</DropdownMenu.Item>
			{#if canPost}
				<DropdownMenu.Item onSelect={openPost}>
					<CheckCircleIcon />
					Post conversion
				</DropdownMenu.Item>
			{/if}
			{#if canCancel}
				<DropdownMenu.Item variant="destructive" onSelect={openCancel}>
					<BanIcon />
					Cancel conversion
				</DropdownMenu.Item>
			{/if}
		</DropdownMenu.Content>
	</DropdownMenu.Root>

	<StockConversionDetailDialog stockConversionId={stockConversion.id} bind:open={isDetailOpen} />
	<StockConversionPostAlertDialog {stockConversion} bind:open={isPostOpen} onPosted={onChanged} />
	<StockConversionCancelDialog {stockConversion} bind:open={isCancelOpen} onCancelled={onChanged} />
</div>
