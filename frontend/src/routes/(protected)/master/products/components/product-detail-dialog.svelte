<script lang="ts">
	import { getProduct, type Product } from '$lib/api/products.js';
	import { ApiError } from '$lib/api/types.js';
	import DetailField from '$lib/components/detail-field.svelte';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import StatusBadge from '$lib/components/status-badge.svelte';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import { formatDateTime } from '$lib/formatters.js';
	import { t } from '$lib/i18n/locale.svelte.js';

	let {
		productId,
		open = $bindable(false)
	}: {
		productId: number;
		open: boolean;
	} = $props();

	let product = $state<Product | null>(null);
	let isLoading = $state(false);
	let errorMessage = $state<string | null>(null);
	let latestRequestId = 0;

	$effect(() => {
		if (!open) return;
		void loadProductDetail(productId);
	});

	async function loadProductDetail(id: number) {
		const requestId = ++latestRequestId;
		isLoading = true;
		errorMessage = null;

		try {
			const result = await getProduct(id);
			if (requestId !== latestRequestId) return;
			product = result;
		} catch (error) {
			if (requestId !== latestRequestId) return;
			product = null;
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
		return t('master.products.errorLoadDetail');
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="sm:max-w-lg">
		<Dialog.Header>
			<Dialog.Title>{t('master.products.detailTitle')}</Dialog.Title>
			<Dialog.Description>{t('master.products.detailDesc')}</Dialog.Description>
		</Dialog.Header>

		{#if isLoading}
			<div class="grid gap-3">
				<Skeleton class="h-5 w-40" />
				<Skeleton class="h-5 w-52" />
				<Skeleton class="h-5 w-32" />
				<Skeleton class="h-5 w-48" />
			</div>
		{:else if errorMessage}
			<ErrorAlert title={t('common.loadFailed')} message={errorMessage} />
		{:else if product}
			<div class="grid gap-4">
				<DetailField label={t('common.name')} value={product.name} />
				<DetailField
					label={t('master.products.category')}
					value={product.category_name || t('master.products.noCategory')}
				/>

				<div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
					<DetailField label={t('common.status')}>
						<StatusBadge active={product.is_active} />
					</DetailField>
					<DetailField label={t('common.id')} value={product.id} />
					<DetailField label={t('common.createdAt')} value={formatDateTime(product.created_at)} />
					<DetailField label={t('common.updatedAt')} value={formatDateTime(product.updated_at)} />
				</div>
			</div>
		{/if}
	</Dialog.Content>
</Dialog.Root>
