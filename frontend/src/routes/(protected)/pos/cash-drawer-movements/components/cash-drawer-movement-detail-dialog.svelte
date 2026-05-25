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
		return direction === 'in' ? 'In' : 'Out';
	}

	function getErrorMessage(error: unknown) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return 'Failed to load movement details.';
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="max-h-[90vh] overflow-y-auto sm:max-w-3xl">
		<Dialog.Header>
			<Dialog.Title>Cash Movement Details</Dialog.Title>
			<Dialog.Description>Manual cash drawer movement and cancellation information.</Dialog.Description>
		</Dialog.Header>

		{#if isLoading}
			<div class="grid gap-3">
				<Skeleton class="h-5 w-48" />
				<Skeleton class="h-5 w-36" />
				<Skeleton class="h-5 w-52" />
				<Skeleton class="h-28 w-full" />
			</div>
		{:else if errorMessage}
			<ErrorAlert title="Failed to load details." message={errorMessage} />
		{:else if movement}
			<div class="grid gap-5">
				<div class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
					<DetailField label="Movement No" value={movement.movement_no} />
					<DetailField label="Status">
						<CashDrawerMovementStatusBadge status={movement.status} />
					</DetailField>
					<DetailField label="Type">
						<CashDrawerMovementTypeBadge type={movement.movement_type} />
					</DetailField>
					<DetailField label="Direction" value={getDirectionLabel(movement.direction)} />
					<DetailField label="Amount" value={formatIdr(movement.amount)} />
					<DetailField label="Occurred At" value={formatDateTime(movement.occurred_at)} />
					<DetailField label="Session" value={movement.session_no} />
					<DetailField label="Register" value={movement.cash_register_code} />
					<DetailField label="Created By" value={movement.created_by_name} />
					<DetailField label="Cancelled At" value={formatDateTime(movement.cancelled_at)} />
					<DetailField label="Cancelled By" value={movement.cancelled_by_name || '-'} />
					<DetailField label="Updated At" value={formatDateTime(movement.updated_at)} />
				</div>

				<div class="grid grid-cols-1 gap-3 sm:grid-cols-3">
					<DetailField label="Reason" value={movement.reason || '-'} />
					<DetailField label="Notes" value={movement.notes || '-'} />
					<DetailField label="Cancel Reason" value={movement.cancel_reason || '-'} />
				</div>
			</div>
		{/if}
	</Dialog.Content>
</Dialog.Root>
