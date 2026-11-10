import { apiGet, apiPost } from './client.js';
import type { CheckoutInput, CheckoutResult } from './storefront.js';

export type MidtransChargeItem = {
	product_sku_id: number;
	quantity: number;
};

export type CreateMidtransChargePayload = {
	cash_register_session_id: number;
	member_id?: number | null;
	notes?: string | null;
	items: MidtransChargeItem[];
};

export type MidtransChargeResult = {
	order_id: string;
	snap_token: string | null;
	redirect_url: string | null;
	gross_amount: number;
	expires_at: string | null;
};

export type MidtransPaymentStatus = {
	order_id: string;
	source_type: string;
	gross_amount: number;
	transaction_status: string;
	fraud_status: string | null;
	payment_type: string | null;
	snap_token: string | null;
	redirect_url: string | null;
	sale_id: number | null;
	online_order_id: number | null;
	expires_at: string | null;
	settled_at: string | null;
};

/** Lunas menurut Midtrans: `settlement`, atau `capture` dengan fraud `accept`. */
export function isMidtransSettled(transactionStatus: string, fraudStatus: string | null): boolean {
	if (transactionStatus === 'settlement') return true;
	return transactionStatus === 'capture' && fraudStatus === 'accept';
}

/** Status terminal tanpa penjualan: kedaluwarsa, dibatalkan, ditolak, gagal. */
export function isMidtransTerminal(transactionStatus: string, fraudStatus: string | null): boolean {
	if (isMidtransSettled(transactionStatus, fraudStatus)) return true;
	return ['expire', 'cancel', 'deny', 'failure'].includes(transactionStatus);
}

/** Charge Midtrans dari kasir POS: pembayaran pending, stok belum berkurang. */
export function createMidtransCharge(payload: CreateMidtransChargePayload) {
	return apiPost<MidtransChargeResult, CreateMidtransChargePayload>(
		'/payments/midtrans/charges',
		payload
	);
}

/** Polling status pembayaran kasir. `refresh=true` menantang Midtrans dulu. */
export function getMidtransPayment(orderId: string, refresh = true) {
	return apiGet<MidtransPaymentStatus>(`/payments/midtrans/${orderId}`, {
		params: { refresh: refresh ? 'true' : undefined }
	});
}

/**
 * Checkout online via Midtrans: jalur publik yang sama dengan transfer,
 * hanya `payment_method` yang dipaksa `midtrans`. Respons membawa
 * `snap_token`/`redirect_url`, bukan `payment_info`.
 */
export function createMidtransOnlineCheckout(
	input: CheckoutInput,
	token: string | null = null
): Promise<CheckoutResult> {
	return apiPost<CheckoutResult, CheckoutInput>('/public/orders', input, {
		authToken: token,
		retryOnUnauthorized: false
	});
}
