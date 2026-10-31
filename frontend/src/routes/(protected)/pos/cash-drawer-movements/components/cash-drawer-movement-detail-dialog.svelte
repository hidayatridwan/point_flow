<script lang="ts">
	import { getCashDrawerMovement, type CashDrawerMovement } from '$lib/api/cash-registers.js';
	import { ApiError } from '$lib/api/types.js';
	import DetailField from '$lib/components/detail-field.svelte';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import { formatDateTime, formatIdr } from '$lib/formatters.js';
	import CashDrawerMovementStatusBadge from './cash-drawer-movement-status-badge.svelte';
	import CashDrawerMovementTypeBadge from './cash-drawer-movement-type-badge.svelte';
	import { t } from '$lib/i18n/locale.svelte.js';

	let {
		movementId,
		open = $bindable(false)
	}: {
		movementId: number;
		open: boolean;
	} = $props();

	let movement = $state<CashDrawerMovement | null>(null);
	let isLoading = $state(false);
	let errorMessage = $state<string | null>(null);
	let latestRequestId = 0;

	$effect(() => {
		if (!open) return;
		void loadMovement(movementId);
	});

	async function loadMovement(id: number) {
		const requestId = ++latestRequestId;
		isLoading = true;
		errorMessage = null;

		try {
			const result = await getCashDrawerMovement(id);
			if (requestId !== latestRequestId) return;
			movement = result;
		} catch (error) {
			if (requestId !== latestRequestId) return;
			movement = null;
			errorMessage = getErrorMessage(error);
		} finally {
			if (requestId === latestRequestId) {
				isLoading = false;
			}
		}
	}

	function getDirectionLabel(direction: string) {
		return direction === 'in'
			? t('inventory.stockMovements.directionIn')
			: t('inventory.stockMovements.directionOut');
	}

	function getErrorMessage(error: unknown) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return t('pos.cashMovements.detail.loadFailed');
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="max-h-[90vh] overflow-y-auto sm:max-w-3xl">
		<Dialog.Header>
			<Dialog.Title>{t('pos.cashMovements.detail.title')}</Dialog.Title>
			<Dialog.Description>{t('pos.cashMovements.detail.desc')}</Dialog.Description>
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
		{:else if movement}
			<div class="grid gap-5">
				<div class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
					<DetailField label={t('common.movementNo')} value={movement.movement_no} />
					<DetailField label={t('common.status')}>
						<CashDrawerMovementStatusBadge status={movement.status} />
					</DetailField>
					<DetailField label={t('common.direction')}>
						<CashDrawerMovementTypeBadge type={movement.movement_type} />
					</DetailField>
					<DetailField
						label={t('common.direction')}
						value={getDirectionLabel(movement.direction)}
					/>
					<DetailField label={t('common.amount')} value={formatIdr(movement.amount)} />
					<DetailField
						label={t('common.occurredAt')}
						value={formatDateTime(movement.occurred_at)}
					/>
					<DetailField label={t('pos.cashMovements.drawerSession')} value={movement.session_no} />
					<DetailField label={t('common.register')} value={movement.cash_register_code} />
					<DetailField label={t('common.createdBy')} value={movement.created_by_name} />
					<DetailField
						label={t('common.cancelledAt')}
						value={formatDateTime(movement.cancelled_at)}
					/>
					<DetailField label={t('common.cancelledBy')} value={movement.cancelled_by_name || '-'} />
					<DetailField label={t('common.updatedAt')} value={formatDateTime(movement.updated_at)} />
				</div>

				<div class="grid grid-cols-1 gap-3 sm:grid-cols-3">
					<DetailField label={t('common.cancelReason')} value={movement.reason || '-'} />
					<DetailField label={t('common.notes')} value={movement.notes || '-'} />
					<DetailField label={t('common.cancelReason')} value={movement.cancel_reason || '-'} />
				</div>
			</div>
		{/if}
	</Dialog.Content>
</Dialog.Root>
