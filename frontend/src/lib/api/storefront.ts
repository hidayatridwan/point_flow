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
	payment_mode: string;
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
	payment_method: 'pay_at_store' | 'transfer' | 'midtrans';
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
	payment_method: 'pay_at_store' | 'transfer' | 'midtrans';
	total_amount: number;
	items: CheckoutItemResult[];
	payment_info: CheckoutPaymentInfo | null;
	snap_token: string | null;
	redirect_url: string | null;
	midtrans_order_id: string | null;
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

export type MemberProfile = {
	id: number;
	member_no: string;
	full_name: string;
	phone: string;
	email: string | null;
	address: string | null;
	joined_at: string;
	is_active: boolean;
	notes: string | null;
	created_at: string;
	updated_at: string;
};

export type MemberLoginResponse = {
	access_token: string;
	token_type: string;
	expires_in: number;
	member: MemberProfile;
};

export type MemberRefreshResponse = MemberLoginResponse;

export async function memberLogin(phone: string, password: string): Promise<MemberLoginResponse> {
	return apiPost<MemberLoginResponse>(
		'/members/auth/login',
		{ phone, password },
		{
			authToken: null,
			retryOnUnauthorized: false
		}
	);
}

export async function refreshMemberSession(): Promise<string | null> {
	try {
		const session = await apiPost<MemberRefreshResponse>('/members/auth/refresh', undefined, {
			authToken: null,
			retryOnUnauthorized: false
		});
		return session.access_token;
	} catch {
		return null;
	}
}

export async function memberLogout(): Promise<void> {
	try {
		await apiPost('/members/auth/logout', undefined, {
			authToken: null,
			retryOnUnauthorized: false
		});
	} catch {
		// Cookie kedaluwarsa duluan: anggap sudah logout.
	}
}

export function getMemberProfile(token: string | null): Promise<MemberProfile> {
	return apiGet<MemberProfile>('/members/auth/me', {
		authToken: token,
		retryOnUnauthorized: false
	});
}

export function changeMemberPassword(
	token: string | null,
	current_password: string,
	new_password: string
): Promise<void> {
	return apiPost<void>(
		'/members/auth/change-password',
		{
			current_password,
			new_password
		},
		{
			authToken: token,
			retryOnUnauthorized: false
		}
	);
}

export function postPublicOrder(
	input: CheckoutInput,
	token: string | null = null
): Promise<CheckoutResult> {
	// Member yang login teratribusi otomatis via token; guest tanpa token.
	return apiPost<CheckoutResult, CheckoutInput>('/public/orders', input, {
		authToken: token,
		retryOnUnauthorized: false
	});
}

export type { ApiEnvelope };
