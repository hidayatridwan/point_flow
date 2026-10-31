<script lang="ts">
	import { getSupplier, type Supplier } from '$lib/api/suppliers.js';
	import { ApiError } from '$lib/api/types.js';
	import DetailField from '$lib/components/detail-field.svelte';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import StatusBadge from '$lib/components/status-badge.svelte';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import { formatDateTime } from '$lib/formatters.js';
	import { t } from '$lib/i18n/locale.svelte.js';

	let {
		supplierId,
		open = $bindable(false)
	}: {
		supplierId: number;
		open: boolean;
	} = $props();

	let supplier = $state<Supplier | null>(null);
	let isLoading = $state(false);
	let errorMessage = $state<string | null>(null);
	let latestRequestId = 0;

	$effect(() => {
		if (!open) return;
		void loadSupplierDetail(supplierId);
	});

	async function loadSupplierDetail(id: number) {
		const requestId = ++latestRequestId;
		isLoading = true;
		errorMessage = null;

		try {
			const result = await getSupplier(id);
			if (requestId !== latestRequestId) return;
			supplier = result;
		} catch (error) {
			if (requestId !== latestRequestId) return;
			supplier = null;
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
		return t('master.suppliers.errorLoadDetail2');
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="sm:max-w-lg">
		<Dialog.Header>
			<Dialog.Title>{t('master.suppliers.detailDialogTitle')}</Dialog.Title>
			<Dialog.Description>{t('master.suppliers.detailDialogDesc')}</Dialog.Description>
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
		{:else if supplier}
			<div class="grid gap-4">
				<DetailField label={t('common.name')} value={supplier.name} />
				<div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
					<DetailField label={t('master.suppliers.contactName')} value={supplier.contact_name} />
					<DetailField label={t('common.phone')} value={supplier.phone} />
					<DetailField label={t('common.email')} value={supplier.email} />
					<DetailField label={t('common.status')}>
						<StatusBadge active={supplier.is_active} />
					</DetailField>
				</div>
				<DetailField label={t('common.address')}>
					<p class="text-sm leading-relaxed">{supplier.address || '-'}</p>
				</DetailField>
				<div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
					<DetailField label={t('common.id')} value={supplier.id} />
					<DetailField label={t('common.createdAt')} value={formatDateTime(supplier.created_at)} />
					<DetailField label={t('common.updatedAt')} value={formatDateTime(supplier.updated_at)} />
				</div>
			</div>
		{/if}
	</Dialog.Content>
</Dialog.Root>
