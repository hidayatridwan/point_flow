<script lang="ts">
	import { getEmployee, type Employee } from '$lib/api/employees.js';
	import { ApiError } from '$lib/api/types.js';
	import DetailField from '$lib/components/detail-field.svelte';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import StatusBadge from '$lib/components/status-badge.svelte';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import { formatDateTime, formatIdr } from '$lib/formatters.js';
	import { t } from '$lib/i18n/locale.svelte.js';

	let {
		employeeId,
		open = $bindable(false)
	}: {
		employeeId: number;
		open: boolean;
	} = $props();

	let employee = $state<Employee | null>(null);
	let isLoading = $state(false);
	let errorMessage = $state<string | null>(null);
	let latestRequestId = 0;

	$effect(() => {
		if (!open) return;
		void loadEmployeeDetail(employeeId);
	});

	async function loadEmployeeDetail(id: number) {
		const requestId = ++latestRequestId;
		isLoading = true;
		errorMessage = null;

		try {
			const result = await getEmployee(id);
			if (requestId !== latestRequestId) return;
			employee = result;
		} catch (error) {
			if (requestId !== latestRequestId) return;
			employee = null;
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
		return t('master.employees.errorLoadDetail');
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="sm:max-w-lg">
		<Dialog.Header>
			<Dialog.Title>{t('master.employees.detailTitle')}</Dialog.Title>
			<Dialog.Description>{t('master.employees.detailDesc')}</Dialog.Description>
		</Dialog.Header>

		{#if isLoading}
			<div class="grid gap-3">
				<Skeleton class="h-5 w-48" />
				<Skeleton class="h-5 w-40" />
				<Skeleton class="h-20 w-full" />
				<Skeleton class="h-5 w-32" />
			</div>
		{:else if errorMessage}
			<ErrorAlert title={t('common.loadFailed')} message={errorMessage} />
		{:else if employee}
			<div class="grid gap-4">
				<DetailField label={t('common.name')} value={employee.name} />
				<div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
					<DetailField label={t('master.employees.position')} value={employee.position} />
					<DetailField label={t('common.phone')} value={employee.phone} />
					<DetailField
						label={t('master.employees.baseSalary')}
						value={formatIdr(employee.base_salary)}
					/>
					<DetailField label={t('common.status')}>
						<StatusBadge active={employee.is_active} />
					</DetailField>
				</div>
				<DetailField label={t('common.address')}>
					<p class="text-sm leading-relaxed">{employee.address || '-'}</p>
				</DetailField>
				<div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
					<DetailField label={t('common.id')} value={employee.id} />
					<DetailField label={t('common.createdAt')} value={formatDateTime(employee.created_at)} />
					<DetailField label={t('common.updatedAt')} value={formatDateTime(employee.updated_at)} />
				</div>
			</div>
		{/if}
	</Dialog.Content>
</Dialog.Root>
