<script lang="ts">
	import { getProduct, uploadProductPhoto, type Product } from '$lib/api/products.js';
	import { ApiError } from '$lib/api/types.js';
	import DetailField from '$lib/components/detail-field.svelte';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import StatusBadge from '$lib/components/status-badge.svelte';
	import { Button } from '$lib/components/ui/button/index.js';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import { Spinner } from '$lib/components/ui/spinner/index.js';
	import UploadIcon from '@lucide/svelte/icons/upload';
	import { formatDateTime } from '$lib/formatters.js';
	import { toast } from 'svelte-sonner';
	import { t } from '$lib/i18n/locale.svelte.js';

	let {
		productId,
		open = $bindable(false),
		onUpdated
	}: {
		productId: number;
		open: boolean;
		onUpdated?: (product: Product) => void;
	} = $props();

	let product = $state<Product | null>(null);
	let isLoading = $state(false);
	let errorMessage = $state<string | null>(null);
	let isUploading = $state(false);
	let uploadError = $state<string | null>(null);
	let latestRequestId = 0;
	let photoInput = $state<HTMLInputElement | null>(null);

	$effect(() => {
		if (!open) return;
		uploadError = null;
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

	async function handlePhotoSelected(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const file = input.files?.[0];
		input.value = '';
		if (!file || isUploading || !product) return;

		if (file.size > 2 * 1024 * 1024) {
			uploadError = t('master.products.photoTooLarge');
			return;
		}

		isUploading = true;
		uploadError = null;

		try {
			await uploadProductPhoto(product.id, file);
			const updated = await getProduct(product.id);
			product = updated;
			onUpdated?.(updated);
			toast.success(t('master.products.photoUploadedToast'));
		} catch (error) {
			if (error instanceof ApiError) uploadError = error.message;
			else if (error instanceof Error) uploadError = error.message;
			else uploadError = t('master.products.errorUploadPhoto');
		} finally {
			isUploading = false;
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
				<div class="flex items-start gap-4">
					{#if product.photo_url}
						<img
							src={product.photo_url}
							alt={product.name}
							class="h-24 w-24 rounded-md border object-cover"
						/>
					{:else}
						<div
							class="flex h-24 w-24 items-center justify-center rounded-md border bg-muted text-xs text-muted-foreground"
						>
							{t('master.products.noPhoto')}
						</div>
					{/if}
					<div class="grid gap-2">
						<Button
							type="button"
							variant="outline"
							size="sm"
							disabled={isUploading}
							onclick={() => photoInput?.click()}
						>
							{#if isUploading}
								<Spinner />
							{:else}
								<UploadIcon />
							{/if}
							{isUploading
								? t('master.products.uploadingPhoto')
								: t('master.products.uploadPhotoBtn')}
						</Button>
						{#if uploadError}
							<p class="text-xs text-destructive">{uploadError}</p>
						{/if}
						<input
							type="file"
							accept="image/jpeg,image/png,image/webp"
							class="hidden"
							bind:this={photoInput}
							onchange={handlePhotoSelected}
						/>
					</div>
				</div>
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
