import { apiGet, apiGetEnvelope, apiPost } from './client.js';
import type { ApiEnvelope, PaginationMeta } from './types.js';

export type PublicProduct = {
	product_sku_id: number;
	product_id: number;
	product_name: string;
	product_sku: string;
	unit: string;
	selling_price: number;
	stock: number;
	category_id: number | null;
	category_name: string | null;
	photo_url: string | null;
};

export type PublicCategory = {
	id: number;
	name: string;
};

export type StoreInfo = {
	store_name: string;
	operating_hours: string;
	bank_accounts: string;
	ewallet_numbers: string;
	qris_image_url: string | null;
	delivery_fee: number;
	shop_whatsapp: string;
};

export type CheckoutItemInput = {
	product_sku_id: number;
	quantity: number;
};

export type CheckoutInput = {
	customer_name: string;
	customer_phone: string;
	fulfillment_method: 'pickup' | 'delivery';
	delivery_address?: string;
	payment_method: 'pay_at_store' | 'transfer';
	notes?: string;
	items: CheckoutItemInput[];
};

export type CheckoutItemResult = {
	product_sku: string;
	product_name: string;
	quantity: number;
	unit_price: number;
	line_total: number;
};

export type CheckoutPaymentInfo = {
	bank_accounts: string;
	ewallet_numbers: string;
	qris_image_url: string | null;
	total_amount: number;
};

export type CheckoutResult = {
	order_id: number;
	order_no: string;
	fulfillment_method: 'pickup' | 'delivery';
	payment_method: 'pay_at_store' | 'transfer';
	total_amount: number;
	items: CheckoutItemResult[];
	payment_info: CheckoutPaymentInfo | null;
};

export type PublicProductList = {
	items: PublicProduct[];
	meta: PaginationMeta;
};

export async function getPublicProducts(
	params: {
		search?: string;
		category_id?: number;
		limit?: number;
		offset?: number;
	} = {}
): Promise<PublicProductList> {
	const envelope = await apiGetEnvelope<PublicProduct[], PaginationMeta>('/public/products', {
		params
	});
	return {
		items: envelope.data ?? [],
		meta: envelope.meta ?? { page: 1, limit: 50, total_records: 0, total_pages: 0 }
	};
}

export function getPublicCategories(): Promise<PublicCategory[]> {
	return apiGet<PublicCategory[]>('/public/categories');
}

export function getPublicStoreInfo(): Promise<StoreInfo> {
	return apiGet<StoreInfo>('/public/store-info');
}

export function postPublicOrder(input: CheckoutInput): Promise<CheckoutResult> {
	return apiPost<CheckoutResult, CheckoutInput>('/public/orders', input);
}

export type { ApiEnvelope };
