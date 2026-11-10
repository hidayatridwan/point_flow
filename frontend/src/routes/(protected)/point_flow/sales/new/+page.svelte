<script lang="ts">
	import BanknoteIcon from '@lucide/svelte/icons/banknote';
	import LayoutGridIcon from '@lucide/svelte/icons/layout-grid';
	import ListIcon from '@lucide/svelte/icons/list';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import SearchIcon from '@lucide/svelte/icons/search';
	import ShoppingCartIcon from '@lucide/svelte/icons/shopping-cart';
	import TrashIcon from '@lucide/svelte/icons/trash';
	import UserPlusIcon from '@lucide/svelte/icons/user-plus';
	import XIcon from '@lucide/svelte/icons/x';
	import {
		clearStoredActiveCashRegisterSessionId,
		getStoredActiveCashRegisterSessionId,
		listCashRegisterSessions,
		setStoredActiveCashRegisterSessionId,
		type CashRegisterSession
	} from '$lib/api/cash-registers.js';
	import { type Member } from '$lib/api/members.js';
	import {
		createMidtransCharge,
		getMidtransPayment,
		isMidtransSettled,
		isMidtransTerminal
	} from '$lib/api/payments.js';
	import { listProductSkus, type ProductSku } from '$lib/api/product-skus.js';
	import { createSale, getSale, type SaleDetail } from '$lib/api/sales.js';
	import { getPublicStoreInfo } from '$lib/api/storefront.js';
	import { loadMidtransSnap } from '$lib/midtrans-snap.js';
	import MemberPickerDialog from '../components/member-picker-dialog.svelte';
	import { ApiError } from '$lib/api/types.js';
	import ErrorAlert from '$lib/components/error-alert.svelte';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import { Spinner } from '$lib/components/ui/spinner/index.js';
	import { Textarea } from '$lib/components/ui/textarea/index.js';
	import { formatIdr, formatNumber } from '$lib/formatters.js';
	import { cn } from '$lib/utils.js';
	import { onDestroy, onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import SaleReceiptDialog from '../components/sale-receipt-dialog.svelte';
	import { t } from '$lib/i18n/locale.svelte.js';

	type CartItem = {
		client_id: string;
		product_sku_id: number;
		product_name: string;
		product_sku: string;
		unit: string;
		stock: number;
		unit_price: number;
		quantity: string;
	};

	type ProductViewMode = 'list' | 'grid';

	let productSkus = $state<ProductSku[]>([]);
	let cartItems = $state<CartItem[]>([]);
	let activeSession = $state<CashRegisterSession | null>(null);
	let skuSearch = $state('');
	let viewMode = $state<ProductViewMode>('grid');
	let paidAmount = $state('');
	let notes = $state('');
	let completedSale = $state<SaleDetail | null>(null);
	let isReceiptOpen = $state(false);
	let selectedMember = $state<Member | null>(null);
	let isMemberPickerOpen = $state(false);
	let isLoadingSkus = $state(true);
	let isLoadingSession = $state(true);
	let isSubmitting = $state(false);
	let errorMessage = $state<string | null>(null);
	let optionsErrorMessage = $state<string | null>(null);
	let sessionErrorMessage = $state<string | null>(null);
	let cartItemSequence = 0;
	let latestOptionsRequestId = 0;
	let latestSessionRequestId = 0;

	type PaymentTab = 'cash' | 'noncash';
	let paymentTab = $state<PaymentTab>('cash');
	let paymentMode = $state('manual');
	const isMidtransMode = $derived(paymentMode === 'midtrans');
	let transferDestination = $state('');
	let receiptNo = $state('');
	let midtransOrderId = $state<string | null>(null);
	let isCreatingCharge = $state(false);
	let isPolling = $state(false);
	let midtransInfo = $state<string | null>(null);
	let pollTimer: ReturnType<typeof setTimeout> | undefined;
	let pollAttempts = 0;
	const maxPollAttempts = 60;
	const pollIntervalMs = 5000;

	const filteredProductSkus = $derived(
		productSkus.filter((productSku) => matchesProductSkuSearch(productSku, skuSearch)).slice(0, 40)
	);
	const subtotalAmount = $derived(cartItems.reduce((total, item) => total + getLineTotal(item), 0));
	const paidAmountValue = $derived(toNumber(paidAmount));
	const changeAmount = $derived(paidAmountValue - subtotalAmount);
	const isPaymentEnough = $derived(cartItems.length > 0 && paidAmountValue >= subtotalAmount);
	const canTransact = $derived(Boolean(activeSession) && !isLoadingSession);

	onMount(() => {
		void loadActiveSession();
		void loadProductSkus();
		void loadPaymentMode();
	});

	onDestroy(() => {
		stopPolling();
	});

	async function loadPaymentMode() {
		try {
			const info = await getPublicStoreInfo();
			paymentMode = info.payment_mode === 'midtrans' ? 'midtrans' : 'manual';
		} catch {
			paymentMode = 'manual';
		}
	}

	async function loadActiveSession() {
		const requestId = ++latestSessionRequestId;
		isLoadingSession = true;
		sessionErrorMessage = null;

		try {
			const result = await listCashRegisterSessions({ limit: 100, status: ['open'] });
			if (requestId !== latestSessionRequestId) return;
			activeSession = resolveActiveSession(result.sessions);

			if (activeSession) {
				setStoredActiveCashRegisterSessionId(activeSession.id);
			} else {
				clearStoredActiveCashRegisterSessionId();
			}
		} catch (error) {
			if (requestId !== latestSessionRequestId) return;
			activeSession = null;
			sessionErrorMessage = getErrorMessage(error, 'Failed to load register session.');
		} finally {
			if (requestId === latestSessionRequestId) {
				isLoadingSession = false;
			}
		}
	}

	async function loadProductSkus() {
		const requestId = ++latestOptionsRequestId;
		isLoadingSkus = true;
		optionsErrorMessage = null;

		try {
			const result = await listProductSkus({ limit: 100 });
			if (requestId !== latestOptionsRequestId) return;
			productSkus = result.productSkus.filter((productSku) => productSku.is_active);
		} catch (error) {
			if (requestId !== latestOptionsRequestId) return;
			productSkus = [];
			optionsErrorMessage = getErrorMessage(error, t('point_flow.newSale.errorLoadSkus'));
		} finally {
			if (requestId === latestOptionsRequestId) {
				isLoadingSkus = false;
			}
		}
	}

	async function handleSubmit(event: SubmitEvent) {
		event.preventDefault();
		if (isSubmitting) return;

		const validationError = getSaleValidationError();
		if (validationError) {
			errorMessage = validationError;
			return;
		}

		isSubmitting = true;
		errorMessage = null;

		try {
			const sale = await createSale({
				cash_register_session_id: activeSession?.id ?? 0,
				member_id: selectedMember?.id ?? null,
				paid_amount: paidAmountValue,
				notes: emptyToNull(notes),
				items: cartItems.map((item) => ({
					product_sku_id: item.product_sku_id,
					quantity: getCartItemQuantity(item)
				}))
			});
			toast.success(t('point_flow.newSale.saleCompletedToast'), {
				description: `${sale.transaction_no} total ${formatIdr(sale.total_amount)}.`
			});
			completedSale = sale;
			isReceiptOpen = true;
			resetSale();
			await loadProductSkus();
		} catch (error) {
			errorMessage = getErrorMessage(error, t('point_flow.newSale.errorCreate'));
		} finally {
			isSubmitting = false;
		}
	}

	async function handleTransferSubmit() {
		if (isSubmitting || isCreatingCharge) return;
		if (!cartItems.length) {
			errorMessage = t('point_flow.newSale.validCartEmpty');
			return;
		}
		if (!activeSession) {
			errorMessage = t('point_flow.newSale.validNoRegister');
			return;
		}
		for (const item of cartItems) {
			const itemError = getCartItemQuantityError(item);
			if (itemError) {
				errorMessage = `${item.product_sku}: ${itemError}`;
				return;
			}
		}
		if (!transferDestination.trim() || !receiptNo.trim()) {
			errorMessage = t('point_flow.newSale.validTransferIncomplete');
			return;
		}

		isSubmitting = true;
		errorMessage = null;
		try {
			const sale = await createSale({
				cash_register_session_id: activeSession?.id ?? 0,
				member_id: selectedMember?.id ?? null,
				paid_amount: subtotalAmount,
				payment_method: 'transfer',
				payment_reference: `${transferDestination.trim()} / ${receiptNo.trim()}`,
				notes: emptyToNull(notes),
				items: cartItems.map((item) => ({
					product_sku_id: item.product_sku_id,
					quantity: getCartItemQuantity(item)
				}))
			});
			toast.success(t('point_flow.newSale.saleCompletedToast'), {
				description: `${sale.transaction_no} total ${formatIdr(sale.total_amount)}.`
			});
			completedSale = sale;
			isReceiptOpen = true;
			transferDestination = '';
			receiptNo = '';
			resetSale();
			await loadProductSkus();
		} catch (error) {
			errorMessage = getErrorMessage(error, t('point_flow.newSale.errorCreate'));
		} finally {
			isSubmitting = false;
		}
	}

	async function handleMidtransCharge() {
		if (isSubmitting || isCreatingCharge) return;
		if (!cartItems.length) {
			errorMessage = t('point_flow.newSale.validCartEmpty');
			return;
		}
		if (!activeSession) {
			errorMessage = t('point_flow.newSale.validNoRegister');
			return;
		}
		for (const item of cartItems) {
			const itemError = getCartItemQuantityError(item);
			if (itemError) {
				errorMessage = `${item.product_sku}: ${itemError}`;
				return;
			}
		}

		isCreatingCharge = true;
		errorMessage = null;
		midtransInfo = null;
		try {
			const charge = await createMidtransCharge({
				cash_register_session_id: activeSession?.id ?? 0,
				member_id: selectedMember?.id ?? null,
				notes: emptyToNull(notes),
				items: cartItems.map((item) => ({
					product_sku_id: item.product_sku_id,
					quantity: getCartItemQuantity(item)
				}))
			});
			if (!charge.snap_token) {
				errorMessage = t('point_flow.newSale.errorCharge');
				return;
			}
			midtransOrderId = charge.order_id;
			await loadMidtransSnap();
			window.snap?.pay(charge.snap_token, {
				onSuccess: () => void checkMidtransOnce(),
				onPending: () => {
					midtransInfo = t('point_flow.newSale.midtransPending');
					startPolling();
				},
				onError: () => {
					stopPolling();
					errorMessage = t('point_flow.newSale.midtransFailed');
				},
				onClose: () => {
					if (!isPolling && midtransOrderId) {
						midtransInfo = t('point_flow.newSale.midtransPending');
						startPolling();
					}
				}
			});
		} catch (error) {
			errorMessage = getErrorMessage(error, t('point_flow.newSale.errorCharge'));
			if (errorMessage === t('point_flow.newSale.errorCharge') && !getMidtransClientKeyReady()) {
				errorMessage = t('point_flow.newSale.snapLoadFailed');
			}
		} finally {
			isCreatingCharge = false;
		}
	}

	function getMidtransClientKeyReady() {
		return typeof document !== 'undefined' && Boolean(window.snap);
	}

	function startPolling() {
		if (isPolling || !midtransOrderId) return;
		isPolling = true;
		pollAttempts = 0;
		midtransInfo = t('point_flow.newSale.midtransPending');
		void pollMidtrans();
	}

	function stopPolling() {
		isPolling = false;
		if (pollTimer) {
			clearTimeout(pollTimer);
			pollTimer = undefined;
		}
	}

	async function checkMidtransOnce() {
		if (!midtransOrderId) return;
		try {
			const status = await getMidtransPayment(midtransOrderId, true);
			await handleMidtransStatus(status.transaction_status, status.fraud_status, status.sale_id);
		} catch (error) {
			errorMessage = getErrorMessage(error, t('point_flow.newSale.errorStatus'));
		}
	}

	async function pollMidtrans() {
		if (!isPolling || !midtransOrderId) return;
		if (pollAttempts >= maxPollAttempts) {
			stopPolling();
			midtransInfo = t('point_flow.newSale.pollingTimeout');
			return;
		}
		pollAttempts += 1;
		try {
			const status = await getMidtransPayment(midtransOrderId, true);
			const settled = await handleMidtransStatus(
				status.transaction_status,
				status.fraud_status,
				status.sale_id
			);
			if (settled) return;
		} catch (error) {
			errorMessage = getErrorMessage(error, t('point_flow.newSale.errorStatus'));
			stopPolling();
			return;
		}
		if (!isPolling) return;
		pollTimer = setTimeout(() => void pollMidtrans(), pollIntervalMs);
	}

	/** Terapkan status Midtrans; true bila polling selesai (lunas/gagal). */
	async function handleMidtransStatus(
		transactionStatus: string,
		fraudStatus: string | null,
		saleId: number | null
	): Promise<boolean> {
		if (isMidtransSettled(transactionStatus, fraudStatus)) {
			stopPolling();
			midtransOrderId = null;
			if (saleId) {
				try {
					const sale = await getSale(saleId);
					toast.success(t('point_flow.newSale.midtransSuccess'), {
						description: `${sale.transaction_no} total ${formatIdr(sale.total_amount)}.`
					});
					completedSale = sale;
					isReceiptOpen = true;
					resetSale();
					await loadProductSkus();
				} catch (error) {
					midtransInfo = t('point_flow.newSale.midtransSuccess');
					errorMessage = getErrorMessage(error, t('point_flow.newSale.errorCreate'));
				}
			} else {
				midtransInfo = t('point_flow.newSale.midtransSuccess');
			}
			return true;
		}
		if (isMidtransTerminal(transactionStatus, fraudStatus)) {
			stopPolling();
			midtransOrderId = null;
			errorMessage = t('point_flow.newSale.midtransFailed');
			midtransInfo = null;
			return true;
		}
		midtransInfo = t('point_flow.newSale.midtransPending');
		return false;
	}

	function addProductSku(productSku: ProductSku) {
		if (!canTransact || isSubmitting || productSku.stock <= 0) return;

		const existingItem = cartItems.find((item) => item.product_sku_id === productSku.id);
		if (existingItem) {
			updateCartItemQuantity(existingItem.client_id, String(getCartItemQuantity(existingItem) + 1));
			return;
		}

		cartItemSequence += 1;
		cartItems = [
			...cartItems,
			{
				client_id: `cart-item-${cartItemSequence}`,
				product_sku_id: productSku.id,
				product_name: productSku.product_name,
				product_sku: productSku.sku,
				unit: productSku.unit_code,
				stock: productSku.stock,
				unit_price: productSku.selling_price,
				quantity: '1'
			}
		];
	}

	function updateCartItemQuantity(clientId: string, value: string) {
		cartItems = cartItems.map((item) => {
			if (item.client_id !== clientId) return item;
			return { ...item, quantity: String(normalizeCartQuantity(value, item.stock)) };
		});
	}

	function removeCartItem(clientId: string) {
		cartItems = cartItems.filter((item) => item.client_id !== clientId);
	}

	function clearCart() {
		cartItems = [];
		paidAmount = '';
		transferDestination = '';
		receiptNo = '';
		notes = '';
		errorMessage = null;
	}

	function resetSale() {
		clearCart();
		skuSearch = '';
		selectedMember = null;
	}

	function removeSelectedMember() {
		selectedMember = null;
	}

	function getSaleValidationError() {
		if (!cartItems.length) return t('point_flow.newSale.validCartEmpty');
		if (!activeSession) return t('point_flow.newSale.validNoRegister');
		for (const item of cartItems) {
			const itemError = getCartItemQuantityError(item);
			if (itemError) return `${item.product_sku}: ${itemError}`;
		}
		if (!Number.isFinite(paidAmountValue) || paidAmountValue < 0) {
			return t('point_flow.newSale.validPaidInvalid');
		}
		if (paidAmountValue < subtotalAmount) {
			return t('point_flow.newSale.validPaidInsufficient');
		}
		return null;
	}

	function resolveActiveSession(sessions: CashRegisterSession[]) {
		const storedSessionId = getStoredActiveCashRegisterSessionId();
		if (storedSessionId) {
			const storedSession = sessions.find((session) => session.id === storedSessionId);
			if (storedSession) return storedSession;
		}
		return sessions[0] ?? null;
	}

	function getCartItemQuantityError(item: CartItem) {
		const quantity = getCartItemQuantity(item);
		if (!Number.isFinite(quantity) || quantity <= 0) return t('point_flow.newSale.validQtyZero');
		if (quantity > item.stock) return t('point_flow.newSale.validQtyExceedsStock');
		return null;
	}

	function getLineTotal(item: CartItem) {
		return getCartItemQuantity(item) * item.unit_price;
	}

	function isCartItemInvalid(item: CartItem) {
		return Boolean(getCartItemQuantityError(item));
	}

	function isProductSkuInCart(productSkuId: number) {
		return cartItems.some((item) => item.product_sku_id === productSkuId);
	}

	function matchesProductSkuSearch(productSku: ProductSku, search: string) {
		const trimmedSearch = search.trim().toLowerCase();
		if (!trimmedSearch) return true;

		return [
			productSku.product_name,
			productSku.sku,
			productSku.barcode,
			productSku.category_name,
			productSku.unit_code
		]
			.filter(Boolean)
			.some((value) => String(value).toLowerCase().includes(trimmedSearch));
	}

	function getCartItemQuantity(item: CartItem) {
		return normalizeCartQuantity(item.quantity, item.stock);
	}

	function normalizeCartQuantity(value: string, stock: number) {
		const parsedQuantity = Number.parseInt(value || '1', 10);
		const quantity = Number.isFinite(parsedQuantity) ? Math.max(1, parsedQuantity) : 1;
		const maxQuantity = Math.floor(stock);
		if (Number.isFinite(maxQuantity) && maxQuantity >= 1) {
			return Math.min(quantity, maxQuantity);
		}
		return quantity;
	}

	function toNumber(value: string) {
		const number = Number(value);
		return Number.isFinite(number) ? number : 0;
	}

	function emptyToNull(value: string) {
		const trimmed = value.trim();
		return trimmed ? trimmed : null;
	}

	function getErrorMessage(error: unknown, fallback: string) {
		if (error instanceof ApiError) return error.message;
		if (error instanceof Error) return error.message;
		return fallback;
	}
</script>

<svelte:head>
	<title>{t('point_flow.newSale.title')}</title>
</svelte:head>

{#if isLoadingSession}
	<div
		class="mb-5 flex items-center gap-2 rounded-md border bg-muted/20 p-4 text-sm text-muted-foreground"
	>
		<Spinner />
		{t('point_flow.newSale.loadingSession')}
	</div>
{:else if sessionErrorMessage}
	<div class="mb-5">
		<ErrorAlert title={t('point_flow.newSale.errorLoadSession')} message={sessionErrorMessage} />
	</div>
{:else if !activeSession}
	<div class="mb-5 rounded-md border bg-muted/20 p-5">
		<div class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
			<div class="min-w-0">
				<h2 class="text-base font-semibold">{t('point_flow.newSale.registerNotOpen')}</h2>
				<p class="mt-1 text-sm text-muted-foreground">
					{t('point_flow.newSale.noActiveRegisterDesc')}
				</p>
			</div>
			<Button href="/point_flow/register-session" class="w-full sm:w-auto">
				<BanknoteIcon />
				{t('point_flow.newSale.openRegisterBtn')}
			</Button>
		</div>
	</div>
{:else}
	<div class="mb-5 rounded-md border bg-muted/20 px-4 py-3 text-sm">
		<span class="text-muted-foreground">{t('point_flow.newSale.activeRegister')}</span>
		<span class="font-medium">{activeSession.cash_register_code}</span>
		<span class="text-muted-foreground">- {activeSession.session_no}</span>
	</div>
{/if}

<form class="min-h-0" onsubmit={handleSubmit}>
	<div class="grid min-h-0 gap-5 lg:grid-cols-[1fr_420px]">
		<section
			class="flex min-h-0 flex-col overflow-hidden rounded-3xl border border-slate-100 bg-white shadow-[0_20px_60px_rgba(15,23,42,0.06)] lg:h-[calc(100dvh-120px)] dark:border-slate-800 dark:bg-slate-950"
			aria-label={t('point_flow.newSale.productSelection')}
		>
			<div class="shrink-0 space-y-3 border-b border-slate-100 p-4 dark:border-slate-800">
				<div class="grid gap-3 md:grid-cols-[minmax(0,1fr)_auto]">
					<div class="relative min-w-0">
						<SearchIcon
							class="absolute top-1/2 left-3 size-4 -translate-y-1/2 text-muted-foreground"
						/>
						<Input
							class="h-11 w-full rounded-2xl border-slate-200 pl-9 text-base shadow-none"
							placeholder={t('point_flow.newSale.searchPlaceholder')}
							disabled={isLoadingSkus || isSubmitting || !canTransact}
							bind:value={skuSearch}
						/>
					</div>

					<div
						class="grid grid-cols-2 rounded-2xl border border-slate-100 bg-slate-50 p-1 dark:border-slate-800 dark:bg-slate-900"
					>
						<Button
							type="button"
							variant={viewMode === 'list' ? 'default' : 'ghost'}
							size="sm"
							class="h-9 rounded-xl"
							aria-pressed={viewMode === 'list'}
							onclick={() => (viewMode = 'list')}
						>
							<ListIcon />
							{t('point_flow.newSale.viewList')}
						</Button>
						<Button
							type="button"
							variant={viewMode === 'grid' ? 'default' : 'ghost'}
							size="sm"
							class="h-9 rounded-xl"
							aria-pressed={viewMode === 'grid'}
							onclick={() => (viewMode = 'grid')}
						>
							<LayoutGridIcon />
							{t('point_flow.newSale.viewGrid')}
						</Button>
					</div>
				</div>

				{#if optionsErrorMessage}
					<ErrorAlert title={t('point_flow.newSale.errorLoadSkus')} message={optionsErrorMessage} />
				{/if}
			</div>

			<div class="max-h-[60vh] min-h-0 flex-1 overflow-y-auto p-4 lg:max-h-none">
				{#if isLoadingSkus}
					<div
						class="flex min-h-40 items-center justify-center gap-2 text-sm text-muted-foreground"
					>
						<Spinner />
						{t('point_flow.newSale.loadingSkus')}
					</div>
				{:else if filteredProductSkus.length > 0}
					{#if viewMode === 'list'}
						<div class="grid gap-2">
							{#each filteredProductSkus as productSku (productSku.id)}
								<div
									class="grid gap-3 rounded-2xl border border-slate-100 bg-slate-50/70 p-3 transition-colors hover:bg-slate-50 sm:grid-cols-[minmax(0,1fr)_auto] sm:items-center dark:border-slate-800 dark:bg-slate-900/40 dark:hover:bg-slate-900"
								>
									<div class="min-w-0">
										<div class="flex flex-wrap items-center gap-x-2 gap-y-1">
											<p class="truncate text-sm font-semibold text-slate-950 dark:text-slate-50">
												{productSku.product_name}
											</p>
											<span
												class="rounded-full bg-white px-2 py-0.5 text-xs text-slate-500 dark:bg-slate-950 dark:text-slate-400"
											>
												{productSku.sku}
											</span>
										</div>
										<p class="mt-1 text-xs text-muted-foreground">
											{productSku.unit_code} · {t('point_flow.newSale.stockLabel')}
											{formatNumber(productSku.stock)} ·
											{formatIdr(productSku.selling_price)}
										</p>
									</div>
									<Button
										type="button"
										variant={isProductSkuInCart(productSku.id) ? 'secondary' : 'outline'}
										class="h-9 rounded-xl sm:w-auto"
										disabled={!canTransact || isSubmitting || productSku.stock <= 0}
										onclick={() => addProductSku(productSku)}
									>
										<PlusIcon />
										{isProductSkuInCart(productSku.id)
											? t('point_flow.newSale.addMore')
											: t('point_flow.newSale.addOne')}
									</Button>
								</div>
							{/each}
						</div>
					{:else}
						<div class="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
							{#each filteredProductSkus as productSku (productSku.id)}
								<div
									class="flex min-h-36 flex-col justify-between gap-4 rounded-2xl border border-slate-100 bg-slate-50/70 p-4 transition-colors hover:bg-slate-50 dark:border-slate-800 dark:bg-slate-900/40 dark:hover:bg-slate-900"
								>
									<div class="min-w-0">
										<p class="line-clamp-2 text-sm font-semibold text-slate-950 dark:text-slate-50">
											{productSku.product_name}
										</p>
										<p class="mt-1 truncate text-xs text-muted-foreground">{productSku.sku}</p>
									</div>

									<div class="grid gap-3">
										<div class="grid grid-cols-2 gap-2 text-xs">
											<div class="rounded-xl bg-white p-2 dark:bg-slate-950">
												<p class="text-muted-foreground">{t('point_flow.newSale.stockLabel')}</p>
												<p class="font-medium text-slate-950 dark:text-slate-50">
													{formatNumber(productSku.stock)}
												</p>
											</div>
											<div class="rounded-xl bg-white p-2 dark:bg-slate-950">
												<p class="text-muted-foreground">{t('point_flow.newSale.priceLabel')}</p>
												<p class="font-medium text-slate-950 dark:text-slate-50">
													{formatIdr(productSku.selling_price)}
												</p>
											</div>
										</div>
										<Button
											type="button"
											variant={isProductSkuInCart(productSku.id) ? 'secondary' : 'outline'}
											class="h-9 rounded-xl"
											disabled={!canTransact || isSubmitting || productSku.stock <= 0}
											onclick={() => addProductSku(productSku)}
										>
											<PlusIcon />
											{isProductSkuInCart(productSku.id) ? 'Add more' : 'Add'}
										</Button>
									</div>
								</div>
							{/each}
						</div>
					{/if}
				{:else}
					<div class="flex min-h-40 items-center justify-center text-sm text-muted-foreground">
						{t('point_flow.newSale.noSkusMatch')}
					</div>
				{/if}
			</div>
		</section>

		<section
			class="flex min-h-0 flex-col overflow-hidden rounded-3xl border border-slate-100 bg-white shadow-[0_20px_60px_rgba(15,23,42,0.06)] lg:h-[calc(100dvh-120px)] dark:border-slate-800 dark:bg-slate-950"
			aria-labelledby="cart-title"
		>
			<div
				class="flex shrink-0 items-center justify-between gap-3 border-b border-slate-100 p-4 dark:border-slate-800"
			>
				<div>
					<h2 id="cart-title" class="text-base font-semibold text-slate-950 dark:text-slate-50">
						{t('point_flow.newSale.cart')}
					</h2>
					<p class="text-sm text-muted-foreground">
						{cartItems.length}
						{t('point_flow.newSale.items')}
					</p>
				</div>
				<Button
					type="button"
					variant="ghost"
					class="rounded-xl text-destructive hover:bg-destructive/10 hover:text-destructive"
					disabled={isSubmitting || cartItems.length === 0}
					onclick={clearCart}
				>
					<TrashIcon />
					{t('common.clear')}
				</Button>
			</div>

			<div class="max-h-[60vh] min-h-0 flex-1 overflow-y-auto p-4 lg:max-h-none">
				{#if cartItems.length > 0}
					<div class="grid gap-3">
						{#each cartItems as item (item.client_id)}
							<div
								class="rounded-2xl border border-slate-100 bg-slate-50/70 p-3 dark:border-slate-800 dark:bg-slate-900/40"
							>
								<div class="flex items-start justify-between gap-3">
									<div class="min-w-0">
										<p class="truncate text-sm font-semibold text-slate-950 dark:text-slate-50">
											{item.product_name}
										</p>
										<p class="text-xs text-muted-foreground">
											{item.product_sku} · {formatIdr(item.unit_price)} · Stock
											{formatNumber(item.stock)}
										</p>
									</div>
									<Button
										type="button"
										variant="ghost"
										size="icon-sm"
										class="size-8 rounded-xl text-destructive hover:bg-destructive/10 hover:text-destructive"
										disabled={isSubmitting}
										onclick={() => removeCartItem(item.client_id)}
										title={t('point_flow.newSale.removeItem')}
									>
										<TrashIcon />
										<span class="sr-only">{t('point_flow.newSale.removeItem')}</span>
									</Button>
								</div>

								<div class="mt-3 flex items-end justify-between gap-3">
									<div>
										<Label class="text-xs text-muted-foreground"
											>{t('point_flow.newSale.qty')}</Label
										>
										<div class="mt-1 flex items-center gap-3">
											<Input
												type="number"
												min="1"
												step="1"
												inputmode="numeric"
												class={cn(
													'h-11 w-24 text-center text-base font-semibold text-slate-950 dark:text-slate-50',
													isCartItemInvalid(item) && 'border-destructive'
												)}
												disabled={isSubmitting}
												value={item.quantity}
												onchange={(event) =>
													updateCartItemQuantity(item.client_id, event.currentTarget.value)}
											/>
											<span class="text-xs text-muted-foreground">{item.unit}</span>
										</div>
									</div>
									<p class="text-sm font-semibold text-slate-950 dark:text-slate-50">
										{formatIdr(getLineTotal(item))}
									</p>
								</div>

								{#if getCartItemQuantityError(item)}
									<p class="mt-2 text-xs text-destructive">{getCartItemQuantityError(item)}</p>
								{/if}
							</div>
						{/each}
					</div>
				{:else}
					<div
						class="flex h-full min-h-40 items-center justify-center text-sm text-muted-foreground"
					>
						{t('point_flow.newSale.emptyCart')}
					</div>
				{/if}
			</div>

			<div
				class="grid shrink-0 gap-4 border-t border-slate-100 bg-slate-50/70 p-4 dark:border-slate-800 dark:bg-slate-900/40"
			>
				<div class="grid gap-2">
					<div class="flex items-center justify-between gap-3 text-sm">
						<span class="text-muted-foreground">{t('point_flow.newSale.subtotal')}</span>
						<span class="font-medium">{formatIdr(subtotalAmount)}</span>
					</div>
					<div class="flex items-center justify-between gap-3 text-sm">
						<span class="text-muted-foreground">{t('point_flow.newSale.total')}</span>
						<span class="text-lg font-semibold">{formatIdr(subtotalAmount)}</span>
					</div>
				</div>

				<div class="grid gap-2">
					<span class="text-xs text-muted-foreground">{t('point_flow.newSale.memberLabel')}</span>
					{#if selectedMember}
						<div
							class="flex items-center justify-between gap-3 rounded-2xl bg-white p-3 dark:bg-slate-950"
						>
							<div class="min-w-0">
								<p class="truncate text-sm font-semibold">{selectedMember.full_name}</p>
								<p class="truncate text-xs text-muted-foreground">
									{selectedMember.member_no} · {selectedMember.phone}
								</p>
							</div>
							<Button
								type="button"
								variant="ghost"
								size="icon-sm"
								class="size-8 shrink-0 rounded-xl text-destructive hover:bg-destructive/10 hover:text-destructive"
								disabled={isSubmitting}
								onclick={removeSelectedMember}
								title={t('point_flow.newSale.removeMember')}
							>
								<XIcon />
								<span class="sr-only">{t('point_flow.newSale.removeMember')}</span>
							</Button>
						</div>
					{:else}
						<Button
							type="button"
							variant="outline"
							class="h-10 rounded-xl bg-white dark:bg-slate-950"
							disabled={isSubmitting || !canTransact}
							onclick={() => (isMemberPickerOpen = true)}
						>
							<UserPlusIcon />
							{t('point_flow.newSale.selectMemberBtn')}
						</Button>
						<p class="text-xs text-muted-foreground">{t('point_flow.newSale.guestBuyer')}</p>
					{/if}
				</div>

				<div
					class="grid grid-cols-2 gap-1 rounded-2xl border border-slate-100 bg-white p-1 dark:border-slate-800 dark:bg-slate-950"
					role="tablist"
					aria-label={t('point_flow.newSale.paymentNonCash')}
				>
					<Button
						type="button"
						role="tab"
						aria-selected={paymentTab === 'cash'}
						variant={paymentTab === 'cash' ? 'default' : 'ghost'}
						class="h-9 rounded-xl"
						onclick={() => (paymentTab = 'cash')}
					>
						{t('point_flow.newSale.paymentCash')}
					</Button>
					<Button
						type="button"
						role="tab"
						aria-selected={paymentTab === 'noncash'}
						variant={paymentTab === 'noncash' ? 'default' : 'ghost'}
						class="h-9 rounded-xl"
						onclick={() => (paymentTab = 'noncash')}
					>
						{t('point_flow.newSale.paymentNonCash')}
					</Button>
				</div>

				{#if paymentTab === 'cash'}
					<div class="space-y-2">
						<Label for="sale-paid-amount" class="whitespace-nowrap"
							>{t('point_flow.newSale.paidAmount')}</Label
						>
						<Input
							id="sale-paid-amount"
							type="number"
							min="0"
							step="1"
							placeholder="0"
							class="h-10 rounded-xl bg-white dark:bg-slate-950"
							disabled={isSubmitting || !canTransact}
							bind:value={paidAmount}
						/>
					</div>

					<div
						class="flex items-center justify-between gap-3 rounded-2xl bg-white p-3 dark:bg-slate-950"
					>
						<div class="flex items-center gap-2 text-sm text-muted-foreground">
							<BanknoteIcon class="size-4" />
							{t('point_flow.newSale.change')}
						</div>
						<p
							class={cn(
								'text-lg font-semibold',
								changeAmount < 0 && cartItems.length > 0
									? 'text-destructive'
									: 'text-emerald-600 dark:text-emerald-400'
							)}
						>
							{formatIdr(Math.max(0, changeAmount))}
						</p>
					</div>
				{:else if isMidtransMode}
					<div class="grid gap-2 rounded-2xl bg-white p-3 dark:bg-slate-950">
						<p class="text-sm font-semibold">{t('point_flow.newSale.midtransTitle')}</p>
						<p class="text-xs text-muted-foreground">{t('point_flow.newSale.midtransDesc')}</p>
						{#if midtransInfo}
							<p class="text-sm text-muted-foreground">{midtransInfo}</p>
						{/if}
						<div class="flex flex-col gap-2">
							<Button
								type="button"
								size="lg"
								class="rounded-2xl"
								disabled={isSubmitting ||
									isCreatingCharge ||
									isPolling ||
									!canTransact ||
									cartItems.length === 0}
								onclick={() => void handleMidtransCharge()}
							>
								{#if isCreatingCharge}
									<Spinner />
								{:else}
									<ShoppingCartIcon />
								{/if}
								{isCreatingCharge
									? t('point_flow.newSale.midtransPaying')
									: t('point_flow.newSale.midtransPayBtn')}
							</Button>
							{#if isPolling && midtransOrderId}
								<div class="flex flex-col gap-2 sm:flex-row">
									<Button
										type="button"
										variant="outline"
										class="rounded-2xl"
										onclick={() => void checkMidtransOnce()}
									>
										{t('point_flow.newSale.checkStatus')}
									</Button>
									<Button type="button" variant="ghost" class="rounded-2xl" onclick={stopPolling}>
										{t('point_flow.newSale.stopPolling')}
									</Button>
								</div>
							{/if}
						</div>
					</div>
				{:else}
					<div class="grid gap-2 rounded-2xl bg-white p-3 dark:bg-slate-950">
						<p class="text-sm font-semibold">{t('point_flow.newSale.transferTitle')}</p>
						<p class="text-xs text-muted-foreground">{t('point_flow.newSale.transferDesc')}</p>
						<div class="space-y-2">
							<Label for="sale-transfer-destination" class="whitespace-nowrap"
								>{t('point_flow.newSale.transferDestination')}</Label
							>
							<Input
								id="sale-transfer-destination"
								placeholder={t('point_flow.newSale.transferDestinationPlaceholder')}
								class="h-10 rounded-xl bg-white dark:bg-slate-950"
								disabled={isSubmitting || !canTransact}
								bind:value={transferDestination}
							/>
						</div>
						<div class="space-y-2">
							<Label for="sale-receipt-no" class="whitespace-nowrap"
								>{t('point_flow.newSale.receiptNo')}</Label
							>
							<Input
								id="sale-receipt-no"
								placeholder={t('point_flow.newSale.receiptNoPlaceholder')}
								class="h-10 rounded-xl bg-white dark:bg-slate-950"
								disabled={isSubmitting || !canTransact}
								bind:value={receiptNo}
							/>
						</div>
						<Button
							type="button"
							size="lg"
							class="rounded-2xl"
							disabled={isSubmitting ||
								!canTransact ||
								cartItems.length === 0 ||
								!transferDestination.trim() ||
								!receiptNo.trim()}
							onclick={() => void handleTransferSubmit()}
						>
							{#if isSubmitting}
								<Spinner />
							{:else}
								<ShoppingCartIcon />
							{/if}
							{isSubmitting
								? t('point_flow.newSale.completing')
								: t('point_flow.newSale.completeTransferBtn')}
						</Button>
					</div>
				{/if}

				<div class="space-y-2">
					<Label for="sale-notes" class="whitespace-nowrap">{t('common.notes')}</Label>
					<Textarea
						id="sale-notes"
						placeholder={t('point_flow.newSale.notesPlaceholder')}
						class="min-h-16 resize-none rounded-xl bg-white dark:bg-slate-950"
						disabled={isSubmitting || !canTransact}
						bind:value={notes}
					/>
				</div>

				{#if errorMessage}
					<ErrorAlert title={t('point_flow.newSale.saleFailed')} message={errorMessage} />
				{/if}

				{#if paymentTab === 'cash'}
					<Button
						type="submit"
						size="lg"
						class="rounded-2xl"
						disabled={isSubmitting || !isPaymentEnough || !canTransact}
					>
						{#if isSubmitting}
							<Spinner />
						{:else}
							<ShoppingCartIcon />
						{/if}
						{isSubmitting
							? t('point_flow.newSale.completing')
							: t('point_flow.newSale.completeBtn')}
					</Button>
				{/if}
			</div>
		</section>
	</div>
</form>

<SaleReceiptDialog saleDetail={completedSale} bind:open={isReceiptOpen} />

<MemberPickerDialog
	bind:open={isMemberPickerOpen}
	onSelected={(member) => (selectedMember = member)}
/>
