<script lang="ts">
	import { getFinancialTransaction, type FinancialTransaction } from '$lib/api/finances.js';
	import { ApiError } from '$lib/api/types.js';
	import DetailField from '$lib/components/detail-field.svelte';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import { formatDateTime, formatIdr } from '$lib/formatters.js';
	import FinancialTransactionStatusBadge from './financial-transaction-status-badge.svelte';
	import FinancialTransactionTypeBadge from './financial-transaction-type-badge.svelte';
	import { t } from '$lib/i18n/locale.svelte.js';

	let {
		transactionId,
		open = $bindable(false)
	}: {
		transactionId: number;
		open: boolean;
	} = $props();

	let transaction = $state<FinancialTransaction | null>(null);
	let isLoading = $state(false);
	let errorMessage = $state<string | null>(null);
	let latestRequestId = 0;

	$effect(() => {
		if (!open) return;
		void loadTransaction(transactionId);
	});

	async function loadTransaction(id: number) {
		const requestId = ++latestRequestId;
		isLoading = true;
		errorMessage = null;

		try {
			const result = await getFinancialTransaction(id);
			if (requestId !== latestRequestId) return;
			transaction = result;
		} catch (error) {
			if (requestId !== latestRequestId) return;
			transaction = null;
			errorMessage = getErrorMessage(error);
		} finally {
			if (requestId === latestRequestId) {
				isLoading = false;
			}
		}
	}

	function getErrorMessage(error: unknown) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return t('finances.transactions.detail.loadFailed');
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="max-h-[90vh] overflow-y-auto sm:max-w-3xl">
		<Dialog.Header>
			<Dialog.Title>{t('finances.transactions.detail.title')}</Dialog.Title>
			<Dialog.Description>{t('finances.transactions.detail.desc')}</Dialog.Description>
		</Dialog.Header>

		{#if isLoading}
			<div class="grid gap-3">
				<Skeleton class="h-5 w-48" />
				<Skeleton class="h-5 w-36" />
				<Skeleton class="h-5 w-52" />
				<Skeleton class="h-28 w-full" />
			</div>
		{:else if errorMessage}
			<ErrorAlert title={t('common.loadFailed')} message={errorMessage} />
		{:else if transaction}
			<div class="grid gap-5">
				<div class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
					<DetailField
						label={t('finances.transactions.colNo')}
						value={transaction.transaction_no}
					/>
					<DetailField label={t('common.status')}>
						<FinancialTransactionStatusBadge status={transaction.status} />
					</DetailField>
					<DetailField label={t('finances.transactions.colType')}>
						<FinancialTransactionTypeBadge type={transaction.transaction_type} />
					</DetailField>
					<DetailField
						label={t('finances.transactions.colCategory')}
						value={transaction.financial_category_name}
					/>
					<DetailField
						label={t('finances.transactions.colAmount')}
						value={formatIdr(transaction.amount)}
					/>
					<DetailField
						label={t('finances.transactions.colCash')}
						value={transaction.is_cash
							? t('finances.transactions.cashYes')
							: t('finances.transactions.cashNo')}
					/>
					<DetailField
						label={t('finances.transactions.colDate')}
						value={formatDateTime(transaction.occurred_at)}
					/>
					{#if transaction.description}
						<DetailField
							label={t('finances.transactions.descriptionLabel')}
							value={transaction.description}
						/>
					{/if}
					{#if transaction.status === 'cancelled'}
						<DetailField
							label={t('finances.transactions.cancel.cancelledAt')}
							value={transaction.cancelled_at ? formatDateTime(transaction.cancelled_at) : '-'}
						/>
						{#if transaction.cancel_reason}
							<DetailField
								label={t('finances.transactions.cancel.cancelReason')}
								value={transaction.cancel_reason}
							/>
						{/if}
					{/if}
				</div>
			</div>
		{/if}
	</Dialog.Content>
</Dialog.Root>
