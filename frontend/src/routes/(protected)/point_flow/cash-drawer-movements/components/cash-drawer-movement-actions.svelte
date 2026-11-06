<script lang="ts">
	import BanIcon from '@lucide/svelte/icons/ban';
	import EllipsisIcon from '@lucide/svelte/icons/ellipsis';
	import EyeIcon from '@lucide/svelte/icons/eye';
	import type { CashDrawerMovement } from '$lib/api/cash-registers.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js';
	import { t } from '$lib/i18n/locale.svelte.js';
	import CashDrawerMovementCancelDialog from './cash-drawer-movement-cancel-dialog.svelte';
	import CashDrawerMovementDetailDialog from './cash-drawer-movement-detail-dialog.svelte';

	let {
		movement,
		onChanged
	}: {
		movement: CashDrawerMovement;
		onChanged?: (movement: CashDrawerMovement) => void | Promise<void>;
	} = $props();

	let isDetailOpen = $state(false);
	let isCancelOpen = $state(false);

	const canCancel = $derived(movement.status === 'completed');

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
					aria-label={t('common.openActions').replace('{docNo}', movement.movement_no)}
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
			{#if canCancel}
				<DropdownMenu.Item variant="destructive" onSelect={openCancel}>
					<BanIcon />
					{t('point_flow.cashMovements.cancel.title')}
				</DropdownMenu.Item>
			{/if}
		</DropdownMenu.Content>
	</DropdownMenu.Root>

	<CashDrawerMovementDetailDialog movementId={movement.id} bind:open={isDetailOpen} />
	<CashDrawerMovementCancelDialog {movement} bind:open={isCancelOpen} onCancelled={onChanged} />
</div>
