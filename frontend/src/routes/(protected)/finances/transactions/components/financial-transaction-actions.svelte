<script lang="ts">
	import BanIcon from '@lucide/svelte/icons/ban';
	import EllipsisIcon from '@lucide/svelte/icons/ellipsis';
	import EyeIcon from '@lucide/svelte/icons/eye';
	import PencilIcon from '@lucide/svelte/icons/pencil';
	import type { FinancialTransaction } from '$lib/api/finances.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js';
	import FinancialTransactionCancelDialog from './financial-transaction-cancel-dialog.svelte';
	import FinancialTransactionDetailDialog from './financial-transaction-detail-dialog.svelte';
	import FinancialTransactionEditDialog from './financial-transaction-edit-dialog.svelte';
	import { t } from '$lib/i18n/locale.svelte.js';

	let {
		transaction,
		onUpdated,
		onCancelled
	}: {
		transaction: FinancialTransaction;
		onUpdated?: (transaction: FinancialTransaction) => void;
		onCancelled?: (transaction: FinancialTransaction) => void;
	} = $props();

	let isDetailOpen = $state(false);
	let isEditOpen = $state(false);
	let isCancelOpen = $state(false);

	const isCompleted = $derived(transaction.status === 'completed');
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
					aria-label={`${t('common.action')}: ${transaction.transaction_no}`}
					title={t('common.action')}
				>
					<EllipsisIcon />
				</Button>
			{/snippet}
		</DropdownMenu.Trigger>
		<DropdownMenu.Content align="end" class="min-w-36">
			<DropdownMenu.Item onSelect={() => (isDetailOpen = true)}>
				<EyeIcon />
				{t('common.detail')}
			</DropdownMenu.Item>
			{#if isCompleted}
				<DropdownMenu.Item onSelect={() => (isEditOpen = true)}>
					<PencilIcon />
					{t('common.edit')}
				</DropdownMenu.Item>
				<DropdownMenu.Item variant="destructive" onSelect={() => (isCancelOpen = true)}>
					<BanIcon />
					{t('finances.transactions.cancelBtn')}
				</DropdownMenu.Item>
			{/if}
		</DropdownMenu.Content>
	</DropdownMenu.Root>

	<FinancialTransactionDetailDialog transactionId={transaction.id} bind:open={isDetailOpen} />
	{#if isCompleted}
		<FinancialTransactionEditDialog {transaction} bind:open={isEditOpen} {onUpdated} />
		<FinancialTransactionCancelDialog {transaction} bind:open={isCancelOpen} {onCancelled} />
	{/if}
</div>
