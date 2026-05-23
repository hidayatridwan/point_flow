<script lang="ts">
	import { getProduct, type Product } from '$lib/api/products.js';
	import { ApiError } from '$lib/api/types.js';
	import DetailField from '$lib/components/detail-field.svelte';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import StatusBadge from '$lib/components/status-badge.svelte';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import { formatDateTime } from '$lib/formatters.js';

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
		return 'Failed to load product details.';
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="sm:max-w-lg">
		<Dialog.Header>
			<Dialog.Title>Product Details</Dialog.Title>
			<Dialog.Description>Latest information from the product master record.</Dialog.Description>
		</Dialog.Header>

		{#if isLoading}
			<div class="grid gap-3">
				<Skeleton class="h-5 w-40" />
				<Skeleton class="h-5 w-52" />
				<Skeleton class="h-5 w-32" />
				<Skeleton class="h-5 w-48" />
			</div>
		{:else if errorMessage}
			<ErrorAlert title="Failed to load details." message={errorMessage} />
		{:else if product}
			<div class="grid gap-4">
				<DetailField label="Name" value={product.name} />
				<DetailField label="Category" value={product.category_name || 'No category'} />

				<div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
					<DetailField label="Status">
						<StatusBadge active={product.is_active} />
					</DetailField>
					<DetailField label="ID" value={product.id} />
					<DetailField label="Created At" value={formatDateTime(product.created_at)} />
					<DetailField label="Updated At" value={formatDateTime(product.updated_at)} />
				</div>
			</div>
		{/if}
	</Dialog.Content>
</Dialog.Root>
