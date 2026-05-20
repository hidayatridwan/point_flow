<script lang="ts">
	import CircleAlertIcon from '@lucide/svelte/icons/circle-alert';
	import {
		getProductCategory,
		type ProductCategory
	} from '$lib/api/product-categories.js';
	import { ApiError } from '$lib/api/types.js';
	import * as Alert from '$lib/components/ui/alert/index.js';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import StatusBadge from './status-badge.svelte';

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

	const dateFormatter = new Intl.DateTimeFormat('id-ID', {
		dateStyle: 'medium',
		timeStyle: 'short'
	});

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

	function formatDate(value: string | null | undefined) {
		if (!value) return '-';

		const date = new Date(value);
		if (Number.isNaN(date.getTime())) return '-';
		return dateFormatter.format(date);
	}

	function getErrorMessage(error: unknown) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return 'Gagal memuat detail product category.';
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Content class="sm:max-w-lg">
		<Dialog.Header>
			<Dialog.Title>Detail Product Category</Dialog.Title>
			<Dialog.Description>Informasi terbaru dari data kategori produk.</Dialog.Description>
		</Dialog.Header>

		{#if isLoading}
			<div class="grid gap-3">
				<Skeleton class="h-5 w-40" />
				<Skeleton class="h-20 w-full" />
				<Skeleton class="h-5 w-32" />
				<Skeleton class="h-5 w-48" />
			</div>
		{:else if errorMessage}
			<Alert.Root variant="destructive">
				<CircleAlertIcon />
				<Alert.Title>Gagal memuat detail.</Alert.Title>
				<Alert.Description>{errorMessage}</Alert.Description>
			</Alert.Root>
		{:else if category}
			<div class="grid gap-4">
				<div class="rounded-md border bg-muted/20 p-3">
					<p class="text-xs font-medium text-muted-foreground">Name</p>
					<p class="mt-1 font-medium">{category.name}</p>
				</div>

				<div class="rounded-md border bg-muted/20 p-3">
					<p class="text-xs font-medium text-muted-foreground">Description</p>
					<p class="mt-1 text-sm leading-relaxed">{category.description || '-'}</p>
				</div>

				<div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
					<div class="rounded-md border bg-muted/20 p-3">
						<p class="text-xs font-medium text-muted-foreground">Status</p>
						<div class="mt-2">
							<StatusBadge active={category.is_active} />
						</div>
					</div>
					<div class="rounded-md border bg-muted/20 p-3">
						<p class="text-xs font-medium text-muted-foreground">ID</p>
						<p class="mt-1 font-medium">{category.id}</p>
					</div>
					<div class="rounded-md border bg-muted/20 p-3">
						<p class="text-xs font-medium text-muted-foreground">Created At</p>
						<p class="mt-1 font-medium">{formatDate(category.created_at)}</p>
					</div>
					<div class="rounded-md border bg-muted/20 p-3">
						<p class="text-xs font-medium text-muted-foreground">Updated At</p>
						<p class="mt-1 font-medium">{formatDate(category.updated_at)}</p>
					</div>
				</div>
			</div>
		{/if}
	</Dialog.Content>
</Dialog.Root>
