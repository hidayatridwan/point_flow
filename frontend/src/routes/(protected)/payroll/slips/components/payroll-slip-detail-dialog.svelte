<script lang="ts">
	import { getPayrollSlip, type PayrollSlip } from '$lib/api/payroll.js';
	import { ApiError } from '$lib/api/types.js';
	import DetailField from '$lib/components/detail-field.svelte';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import { formatDate, formatDateTime, formatIdr } from '$lib/formatters.js';
	import { t } from '$lib/i18n/locale.svelte.js';
	import PayrollSlipStatusBadge from './payroll-slip-status-badge.svelte';

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
		return t('payroll.slips.detail.loadFailed');
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="sm:max-w-lg">
		<Dialog.Header>
			<Dialog.Title>{t('payroll.slips.detail.title')}</Dialog.Title>
			<Dialog.Description>{t('payroll.slips.detail.desc')}</Dialog.Description>
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
			<ErrorAlert title={t('common.loadFailed')} message={errorMessage} />
		{:else if payrollSlip}
			<div class="grid gap-5">
				<div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
					<DetailField label={t('payroll.slips.colSlipNo')} value={payrollSlip.slip_no} />
					<DetailField label={t('payroll.slips.employee')} value={payrollSlip.employee_name} />
					<DetailField
						label={t('master.employees.position')}
						value={payrollSlip.employee_position}
					/>
					<DetailField
						label={t('payroll.slips.periodStart')}
						value={formatDate(payrollSlip.period_start)}
					/>
					<DetailField
						label={t('payroll.slips.periodEnd')}
						value={formatDate(payrollSlip.period_end)}
					/>
					<DetailField
						label={t('common.createdAt')}
						value={formatDateTime(payrollSlip.created_at)}
					/>
				</div>

				<div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
					<DetailField
						label={t('payroll.slips.baseSalary')}
						value={formatIdr(payrollSlip.base_salary)}
					/>
					<DetailField label={t('payroll.slips.bonus')} value={formatIdr(payrollSlip.bonus)} />
					<DetailField
						label={t('payroll.slips.deduction')}
						value={formatIdr(payrollSlip.deduction)}
					/>
					<DetailField
						label={t('payroll.slips.netSalary')}
						value={formatIdr(payrollSlip.net_salary)}
					/>
				</div>

				<div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
					<div class="space-y-1.5">
						<p class="text-sm text-muted-foreground">{t('payroll.slips.colStatus')}</p>
						<PayrollSlipStatusBadge status={payrollSlip.status} />
					</div>
					<DetailField label={t('common.createdBy')} value={payrollSlip.created_by ?? '-'} />
					<DetailField
						label={t('common.updatedAt')}
						value={formatDateTime(payrollSlip.updated_at)}
					/>
				</div>

				{#if payrollSlip.status === 'cancelled'}
					<div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
						<DetailField
							label={t('payroll.slips.cancel.cancelledAt')}
							value={payrollSlip.cancelled_at ? formatDateTime(payrollSlip.cancelled_at) : '-'}
						/>
						<DetailField
							label={t('payroll.slips.cancel.cancelReason')}
							value={payrollSlip.cancel_reason ?? '-'}
						/>
					</div>
				{/if}
			</div>
		{/if}
	</Dialog.Content>
</Dialog.Root>
