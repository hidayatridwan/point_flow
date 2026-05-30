<script lang="ts">
	import { getProductSku, type ProductSku } from '$lib/api/product-skus.js';
	import { ApiError } from '$lib/api/types.js';
	import DetailField from '$lib/components/detail-field.svelte';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import StatusBadge from '$lib/components/status-badge.svelte';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import { formatDateTime, formatIdr, formatNumber } from '$lib/formatters.js';
	import { t } from '$lib/i18n/locale.svelte.js';

	let {
		productSkuId,
		open = $bindable(false)
	}: {
		productSkuId: number;
		open: boolean;
	} = $props();

	let productSku = $state<ProductSku | null>(null);
	let isLoading = $state(false);
	let errorMessage = $state<string | null>(null);
	let latestRequestId = 0;

	$effect(() => {
		if (!open) return;
		void loadProductSkuDetail(productSkuId);
	});

	async function loadProductSkuDetail(id: number) {
		const requestId = ++latestRequestId;
		isLoading = true;
		errorMessage = null;

		try {
			const result = await getProductSku(id);
			if (requestId !== latestRequestId) return;
			productSku = result;
		} catch (error) {
			if (requestId !== latestRequestId) return;
			productSku = null;
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
		return t('master.productSkus.errorLoadDetail2');
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="sm:max-w-2xl">
		<Dialog.Header>
			<Dialog.Title>Product SKU Details</Dialog.Title>
			<Dialog.Description>Latest product SKU, pricing, and stock information.</Dialog.Description>
		</Dialog.Header>

		{#if isLoading}
			<div class="grid gap-3">
				<Skeleton class="h-5 w-48" />
				<Skeleton class="h-5 w-36" />
				<Skeleton class="h-5 w-52" />
				<Skeleton class="h-5 w-32" />
			</div>
		{:else if errorMessage}
			<ErrorAlert title={t('common.loadFailed')} message={errorMessage} />
		{:else if productSku}
			<div class="grid gap-4">
				<div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
					<DetailField label="Product" value={productSku.product_name} />
					<DetailField label="Category" value={productSku.category_name || 'No category'} />
					<DetailField label="SKU" value={productSku.sku} />
					<DetailField label="Barcode" value={productSku.barcode || '-'} />
					<DetailField label="Unit" value={`${productSku.unit_name} (${productSku.unit_code})`} />
					<DetailField label="Contents Qty" value={formatNumber(productSku.contents_qty)} />
				</div>

				<div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
					<DetailField label="Purchase Price" value={formatIdr(productSku.purchase_price)} />
					<DetailField label="Selling Price" value={formatIdr(productSku.selling_price)} />
					<DetailField label="Stock" value={formatNumber(productSku.stock)} />
					<DetailField label="Minimum Stock" value={formatNumber(productSku.minimum_stock)} />
				</div>

				<div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
					<DetailField label="Status">
						<StatusBadge active={productSku.is_active} />
					</DetailField>
					<DetailField label="ID" value={productSku.id} />
					<DetailField label="Created At" value={formatDateTime(productSku.created_at)} />
					<DetailField label="Updated At" value={formatDateTime(productSku.updated_at)} />
				</div>
			</div>
		{/if}
	</Dialog.Content>
</Dialog.Root>
