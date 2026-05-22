<script lang="ts">
	import { getUnit, type Unit } from '$lib/api/units.js';
	import { ApiError } from '$lib/api/types.js';
	import DetailField from '$lib/components/detail-field.svelte';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import StatusBadge from '$lib/components/status-badge.svelte';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import { formatDateTime } from '$lib/formatters.js';

	let {
		unitId,
		open = $bindable(false)
	}: {
		unitId: number;
		open: boolean;
	} = $props();

	let unit = $state<Unit | null>(null);
	let isLoading = $state(false);
	let errorMessage = $state<string | null>(null);
	let latestRequestId = 0;

	$effect(() => {
		if (!open) return;
		void loadUnitDetail(unitId);
	});

	async function loadUnitDetail(id: number) {
		const requestId = ++latestRequestId;
		isLoading = true;
		errorMessage = null;

		try {
			const result = await getUnit(id);
			if (requestId !== latestRequestId) return;
			unit = result;
		} catch (error) {
			if (requestId !== latestRequestId) return;
			unit = null;
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
		return 'Failed to load unit details.';
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="sm:max-w-lg">
		<Dialog.Header>
			<Dialog.Title>Unit Details</Dialog.Title>
			<Dialog.Description>Latest information from the unit record.</Dialog.Description>
		</Dialog.Header>

		{#if isLoading}
			<div class="grid gap-3">
				<Skeleton class="h-5 w-32" />
				<Skeleton class="h-5 w-48" />
				<Skeleton class="h-5 w-32" />
				<Skeleton class="h-5 w-48" />
			</div>
		{:else if errorMessage}
			<ErrorAlert title="Failed to load details." message={errorMessage} />
		{:else if unit}
			<div class="grid gap-4">
				<div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
					<DetailField label="Code" value={unit.code} />
					<DetailField label="Name" value={unit.name} />
					<DetailField label="Decimal Quantity">
						<StatusBadge active={unit.allow_decimal} activeLabel="Allowed" inactiveLabel="Whole" />
					</DetailField>
					<DetailField label="Status">
						<StatusBadge active={unit.is_active} />
					</DetailField>
					<DetailField label="ID" value={unit.id} />
					<DetailField label="Created At" value={formatDateTime(unit.created_at)} />
					<DetailField label="Updated At" value={formatDateTime(unit.updated_at)} />
				</div>
			</div>
		{/if}
	</Dialog.Content>
</Dialog.Root>
