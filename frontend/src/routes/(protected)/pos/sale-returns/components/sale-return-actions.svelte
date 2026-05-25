<script lang="ts">
	import BanIcon from '@lucide/svelte/icons/ban';
	import EllipsisIcon from '@lucide/svelte/icons/ellipsis';
	import EyeIcon from '@lucide/svelte/icons/eye';
	import type { SaleReturn } from '$lib/api/sales.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js';
	import SaleReturnCancelDialog from './sale-return-cancel-dialog.svelte';
	import SaleReturnDetailDialog from './sale-return-detail-dialog.svelte';

	let {
		saleReturn,
		onChanged
	}: {
		saleReturn: SaleReturn;
		onChanged?: (saleReturn: SaleReturn) => void | Promise<void>;
	} = $props();

	let isDetailOpen = $state(false);
	let isCancelOpen = $state(false);

	const canCancel = $derived(saleReturn.status === 'completed');

	function openDetail() {
		isDetailOpen = true;
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
					aria-label={`Open actions for ${saleReturn.return_no}`}
					title="Actions"
				>
					<EllipsisIcon />
				</Button>
			{/snippet}
		</DropdownMenu.Trigger>
		<DropdownMenu.Content align="end" class="min-w-44">
			<DropdownMenu.Item onSelect={openDetail}>
				<EyeIcon />
				Detail
			</DropdownMenu.Item>
			{#if canCancel}
				<DropdownMenu.Item variant="destructive" onSelect={openCancel}>
					<BanIcon />
					Cancel return
				</DropdownMenu.Item>
			{/if}
		</DropdownMenu.Content>
	</DropdownMenu.Root>

	<SaleReturnDetailDialog saleReturnId={saleReturn.id} bind:open={isDetailOpen} />
	<SaleReturnCancelDialog {saleReturn} bind:open={isCancelOpen} onCancelled={onChanged} />
</div>
