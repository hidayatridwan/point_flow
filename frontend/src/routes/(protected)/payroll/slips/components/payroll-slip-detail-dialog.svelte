<script lang="ts">
	import { getPayrollSlip, type PayrollSlip } from '$lib/api/payroll.js';
	import { ApiError } from '$lib/api/types.js';
	import DetailField from '$lib/components/detail-field.svelte';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import { formatDate, formatDateTime, formatIdr } from '$lib/formatters.js';

	let {
		payrollSlipId,
		open = $bindable(false)
	}: {
		payrollSlipId: number;
		open: boolean;
	} = $props();

	let payrollSlip = $state<PayrollSlip | null>(null);
	let isLoading = $state(false);
	let errorMessage = $state<string | null>(null);
	let latestRequestId = 0;

	$effect(() => {
		if (!open) return;
		void loadPayrollSlip(payrollSlipId);
	});

	async function loadPayrollSlip(id: number) {
		const requestId = ++latestRequestId;
		isLoading = true;
		errorMessage = null;

		try {
			const result = await getPayrollSlip(id);
			if (requestId !== latestRequestId) return;
			payrollSlip = result;
		} catch (error) {
			if (requestId !== latestRequestId) return;
			payrollSlip = null;
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
		return 'Failed to load payroll slip details.';
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="sm:max-w-lg">
		<Dialog.Header>
			<Dialog.Title>Payroll Slip Details</Dialog.Title>
			<Dialog.Description
				>Employee payroll period, salary components, and net salary.</Dialog.Description
			>
		</Dialog.Header>

		{#if isLoading}
			<div class="grid gap-3 sm:grid-cols-2">
				<Skeleton class="h-20 w-full" />
				<Skeleton class="h-20 w-full" />
				<Skeleton class="h-20 w-full" />
				<Skeleton class="h-20 w-full" />
				<Skeleton class="h-20 w-full" />
				<Skeleton class="h-20 w-full" />
			</div>
		{:else if errorMessage}
			<ErrorAlert title="Failed to load details." message={errorMessage} />
		{:else if payrollSlip}
			<div class="grid gap-5">
				<div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
					<DetailField label="Slip No" value={payrollSlip.slip_no} />
					<DetailField label="Employee" value={payrollSlip.employee_name} />
					<DetailField label="Position" value={payrollSlip.employee_position} />
					<DetailField label="Period Start" value={formatDate(payrollSlip.period_start)} />
					<DetailField label="Period End" value={formatDate(payrollSlip.period_end)} />
					<DetailField label="Created At" value={formatDateTime(payrollSlip.created_at)} />
				</div>

				<div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
					<DetailField label="Base Salary" value={formatIdr(payrollSlip.base_salary)} />
					<DetailField label="Bonus" value={formatIdr(payrollSlip.bonus)} />
					<DetailField label="Deduction" value={formatIdr(payrollSlip.deduction)} />
					<DetailField label="Net Salary" value={formatIdr(payrollSlip.net_salary)} />
				</div>

				<div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
					<DetailField label="Created By" value={payrollSlip.created_by ?? '-'} />
					<DetailField label="Updated At" value={formatDateTime(payrollSlip.updated_at)} />
				</div>
			</div>
		{/if}
	</Dialog.Content>
</Dialog.Root>
