<script lang="ts">
	import { onMount } from 'svelte';
	import UploadIcon from '@lucide/svelte/icons/upload';
	import {
		getStoreSettings,
		updateStoreSettings,
		uploadQrisImage,
		type StoreSettings
	} from '$lib/api/online-orders.js';
	import { ApiError } from '$lib/api/types.js';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import { Button } from '$lib/components/ui/button/index.js';
	import * as Card from '$lib/components/ui/card/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import { Spinner } from '$lib/components/ui/spinner/index.js';
	import { Textarea } from '$lib/components/ui/textarea/index.js';
	import { formatIdr } from '$lib/formatters.js';
	import { toast } from 'svelte-sonner';
	import { t } from '$lib/i18n/locale.svelte.js';

	let isLoading = $state(true);
	let isSaving = $state(false);
	let isUploading = $state(false);
	let loadError = $state<string | null>(null);
	let saveError = $state<string | null>(null);
	let uploadError = $state<string | null>(null);

	let storeName = $state('');
	let operatingHours = $state('');
	let bankAccounts = $state('');
	let ewalletNumbers = $state('');
	let deliveryFee = $state('0');
	let shopWhatsapp = $state('');
	let qrisImageUrl = $state<string | null>(null);
	let qrisInput = $state<HTMLInputElement | null>(null);

	onMount(() => {
		void loadSettings();
	});

	async function loadSettings() {
		isLoading = true;
		loadError = null;

		try {
			const settings = await getStoreSettings();
			applySettings(settings);
		} catch (error) {
			loadError = getErrorMessage(error, t('config.storeSettings.errorLoad'));
		} finally {
			isLoading = false;
		}
	}

	function applySettings(settings: StoreSettings) {
		storeName = settings.store_name;
		operatingHours = settings.operating_hours;
		bankAccounts = settings.bank_accounts;
		ewalletNumbers = settings.ewallet_numbers;
		deliveryFee = String(settings.delivery_fee);
		shopWhatsapp = settings.shop_whatsapp;
		qrisImageUrl = settings.qris_image_url;
	}

	async function handleSave(event: SubmitEvent) {
		event.preventDefault();
		if (isSaving || isLoading) return;

		const fee = Number(deliveryFee);
		if (!Number.isFinite(fee) || fee < 0) {
			saveError = t('config.storeSettings.invalidDeliveryFee');
			return;
		}

		isSaving = true;
		saveError = null;

		try {
			const settings = await updateStoreSettings({
				store_name: storeName.trim(),
				operating_hours: operatingHours.trim(),
				bank_accounts: bankAccounts.trim(),
				ewallet_numbers: ewalletNumbers.trim(),
				delivery_fee: fee,
				shop_whatsapp: shopWhatsapp.trim()
			});
			applySettings(settings);
			toast.success(t('config.storeSettings.savedToast'));
		} catch (error) {
			saveError = getErrorMessage(error, t('config.storeSettings.errorSave'));
		} finally {
			isSaving = false;
		}
	}

	async function handleQrisSelected(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const file = input.files?.[0];
		input.value = '';
		if (!file || isUploading) return;

		if (file.size > 2 * 1024 * 1024) {
			uploadError = t('config.storeSettings.imageTooLarge');
			return;
		}

		isUploading = true;
		uploadError = null;

		try {
			const result = await uploadQrisImage(file);
			qrisImageUrl = result.url;
			toast.success(t('config.storeSettings.qrisUploadedToast'));
		} catch (error) {
			uploadError = getErrorMessage(error, t('config.storeSettings.errorUpload'));
		} finally {
			isUploading = false;
		}
	}

	function getErrorMessage(error: unknown, fallback: string) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return fallback;
	}
</script>

<svelte:head>
	<title>{t('config.storeSettings.title')} | PointFlow</title>
</svelte:head>

<div class="flex flex-col gap-6">
	<Card.Root>
		<Card.Header>
			<div>
				<Card.Title>{t('config.storeSettings.title')}</Card.Title>
				<Card.Description>{t('config.storeSettings.subtitle')}</Card.Description>
			</div>
		</Card.Header>

		{#if isLoading}
			<Card.Content class="grid gap-4">
				<Skeleton class="h-9 w-full" />
				<Skeleton class="h-9 w-full" />
				<Skeleton class="h-20 w-full" />
				<Skeleton class="h-9 w-40" />
			</Card.Content>
		{:else if loadError}
			<Card.Content>
				<ErrorAlert title={t('config.storeSettings.errorLoad')} message={loadError} />
			</Card.Content>
		{:else}
			<form onsubmit={handleSave}>
				<Card.Content class="grid gap-4">
					<div class="grid gap-2">
						<Label for="store-name">{t('config.storeSettings.storeName')}</Label>
						<Input
							id="store-name"
							bind:value={storeName}
							placeholder={t('config.storeSettings.storeNamePlaceholder')}
							disabled={isSaving}
							maxlength={200}
						/>
					</div>

					<div class="grid gap-2">
						<Label for="operating-hours">{t('config.storeSettings.operatingHours')}</Label>
						<Input
							id="operating-hours"
							bind:value={operatingHours}
							placeholder={t('config.storeSettings.operatingHoursPlaceholder')}
							disabled={isSaving}
							maxlength={500}
						/>
					</div>

					<div class="grid gap-2 sm:grid-cols-2">
						<div class="grid gap-2">
							<Label for="bank-accounts">{t('config.storeSettings.bankAccounts')}</Label>
							<Textarea
								id="bank-accounts"
								bind:value={bankAccounts}
								placeholder={t('config.storeSettings.bankAccountsPlaceholder')}
								disabled={isSaving}
								rows={3}
							/>
						</div>
						<div class="grid gap-2">
							<Label for="ewallet-numbers">{t('config.storeSettings.ewalletNumbers')}</Label>
							<Textarea
								id="ewallet-numbers"
								bind:value={ewalletNumbers}
								placeholder={t('config.storeSettings.ewalletNumbersPlaceholder')}
								disabled={isSaving}
								rows={3}
							/>
						</div>
					</div>

					<div class="grid gap-2 sm:grid-cols-2">
						<div class="grid gap-2">
							<Label for="delivery-fee">{t('config.storeSettings.deliveryFee')}</Label>
							<Input
								id="delivery-fee"
								type="number"
								min="0"
								step="500"
								bind:value={deliveryFee}
								disabled={isSaving}
							/>
							<p class="text-xs text-muted-foreground">
								{t('config.storeSettings.deliveryFeeHint')}: {formatIdr(Number(deliveryFee) || 0)}
							</p>
						</div>
						<div class="grid gap-2">
							<Label for="shop-whatsapp">{t('config.storeSettings.shopWhatsapp')}</Label>
							<Input
								id="shop-whatsapp"
								bind:value={shopWhatsapp}
								placeholder={t('config.storeSettings.shopWhatsappPlaceholder')}
								disabled={isSaving}
								maxlength={30}
								inputmode="tel"
							/>
						</div>
					</div>

					{#if saveError}
						<ErrorAlert title={t('config.storeSettings.errorSave')} message={saveError} />
					{/if}
				</Card.Content>

				<Card.Footer class="flex justify-end border-t px-6 py-4">
					<Button type="submit" disabled={isSaving}>
						{#if isSaving}
							<Spinner />
						{/if}
						{isSaving ? t('config.storeSettings.saving') : t('config.storeSettings.saveBtn')}
					</Button>
				</Card.Footer>
			</form>
		{/if}
	</Card.Root>

	<Card.Root>
		<Card.Header>
			<div>
				<Card.Title>{t('config.storeSettings.qrisTitle')}</Card.Title>
				<Card.Description>{t('config.storeSettings.qrisDesc')}</Card.Description>
			</div>
		</Card.Header>
		<Card.Content class="grid gap-4">
			{#if qrisImageUrl}
				<img
					src={qrisImageUrl}
					alt={t('config.storeSettings.qrisTitle')}
					class="h-48 w-48 rounded-md border object-contain"
				/>
			{:else}
				<p class="text-sm text-muted-foreground">{t('config.storeSettings.qrisEmpty')}</p>
			{/if}

			{#if uploadError}
				<ErrorAlert title={t('config.storeSettings.errorUpload')} message={uploadError} />
			{/if}

			<input
				type="file"
				accept="image/jpeg,image/png,image/webp"
				class="hidden"
				bind:this={qrisInput}
				onchange={handleQrisSelected}
			/>
		</Card.Content>
		<Card.Footer class="flex justify-start border-t px-6 py-4">
			<Button
				type="button"
				variant="outline"
				disabled={isLoading || isUploading}
				onclick={() => qrisInput?.click()}
			>
				{#if isUploading}
					<Spinner />
				{:else}
					<UploadIcon />
				{/if}
				{isUploading
					? t('config.storeSettings.uploading')
					: t('config.storeSettings.uploadQrisBtn')}
			</Button>
		</Card.Footer>
	</Card.Root>
</div>
