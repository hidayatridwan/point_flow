<script lang="ts">
	import { getProductCategory, type ProductCategory } from '$lib/api/product-categories.js';
	import { ApiError } from '$lib/api/types.js';
	import DetailField from '$lib/components/detail-field.svelte';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import StatusBadge from '$lib/components/status-badge.svelte';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import { formatDateTime } from '$lib/formatters.js';

	let {
		categoryId,
		open = $bindable(false)
	}: {
		categoryId: number;
		open: boolean;
	} = $props();

	let category = $state<ProductCategory | null>(null);
	let isLoading = $state(false);
	let errorMessage = $state<string | null>(null);
	let latestRequestId = 0;

	$effect(() => {
		if (!open) return;
		void loadCategoryDetail(categoryId);
	});

	async function loadCategoryDetail(id: number) {
		const requestId = ++latestRequestId;
		isLoading = true;
		errorMessage = null;

		try {
			const result = await getProductCategory(id);
			if (requestId !== latestRequestId) return;
			category = result;
		} catch (error) {
			if (requestId !== latestRequestId) return;
			category = null;
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
		return 'Failed to load product category details.';
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="sm:max-w-lg">
		<Dialog.Header>
			<Dialog.Title>Product Category Details</Dialog.Title>
			<Dialog.Description>Latest information from the product category record.</Dialog.Description>
		</Dialog.Header>

		{#if isLoading}
			<div class="grid gap-3">
				<Skeleton class="h-5 w-40" />
				<Skeleton class="h-20 w-full" />
				<Skeleton class="h-5 w-32" />
				<Skeleton class="h-5 w-48" />
			</div>
		{:else if errorMessage}
			<ErrorAlert title="Failed to load details." message={errorMessage} />
		{:else if category}
			<div class="grid gap-4">
				<DetailField label="Name" value={category.name} />

				<DetailField label="Description">
					<p class="text-sm leading-relaxed">{category.description || '-'}</p>
				</DetailField>

				<div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
					<DetailField label="Status">
						<StatusBadge active={category.is_active} />
					</DetailField>
					<DetailField label="ID" value={category.id} />
					<DetailField label="Created At" value={formatDateTime(category.created_at)} />
					<DetailField label="Updated At" value={formatDateTime(category.updated_at)} />
				</div>
			</div>
		{/if}
	</Dialog.Content>
</Dialog.Root>
