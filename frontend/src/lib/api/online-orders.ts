import { apiGet, apiGetEnvelope, apiPost, apiPut } from './client.js';
import type { PaginationMeta } from './types.js';

export const onlineOrderStatuses = [
	'new',
	'packing',
	'ready',
	'delivering',
	'completed',
	'cancelled'
] as const;

export type OnlineOrderStatus = (typeof onlineOrderStatuses)[number];

export type OnlineOrder = {
	id: number;
	order_no: string;
	customer_name: string;
	customer_phone: string;
	fulfillment_method: string;
	delivery_address: string | null;
	payment_method: string;
	payment_status: string;
	status: OnlineOrderStatus;
	subtotal_amount: number;
	delivery_fee: number;
	total_amount: number;
	notes: string | null;
	sale_id: number | null;
	cancel_reason: string | null;
	member_id: number | null;
};

export type OnlineOrderItem = {
	id: number;
	product_sku_id: number;
	product_name: string;
	product_sku: string;
	unit: string;
	quantity: number;
	unit_price: number;
	line_total: number;
};

export type OnlineOrderDetail = {
	order: OnlineOrder;
	items: OnlineOrderItem[];
};

export type OnlineOrderListParams = {
	page?: number;
	limit?: number;
	search?: string;
	status?: OnlineOrderStatus[];
};

export type OnlineOrderPagination = {
	page: number;
	pageSize: number;
	totalRecords: number;
	totalPages: number;
};

export type OnlineOrderListResult = {
	orders: OnlineOrder[];
	pagination: OnlineOrderPagination;
	requestId: string;
};

export type StoreSettings = {
	store_name: string;
	operating_hours: string;
	bank_accounts: string;
	ewallet_numbers: string;
	qris_image_url: string | null;
	delivery_fee: number;
	shop_whatsapp: string;
	payment_mode: string;
};

export type UpdateStoreSettingsPayload = {
	store_name?: string;
	operating_hours?: string;
	bank_accounts?: string;
	ewallet_numbers?: string;
	delivery_fee?: number;
	shop_whatsapp?: string;
	payment_mode?: string;
};

export type UploadResponse = {
	key: string;
	url: string;
};

export async function listOnlineOrders(
	params: OnlineOrderListParams = {}
): Promise<OnlineOrderListResult> {
	const limit = normalizeLimit(params.limit);
	const page = normalizePage(params.page);
	const search = params.search?.trim();
	const status = normalizeStatuses(params.status);

	const envelope = await apiGetEnvelope<OnlineOrder[], PaginationMeta>('/online-orders', {
		params: {
			limit,
			offset: (page - 1) * limit,
			search,
			status: status.length ? status.join(',') : undefined
		}
	});
	const orders = envelope.data ?? [];
	const meta = envelope.meta ?? getFallbackMeta(page, limit, orders.length);

	return {
		orders,
		pagination: {
			page: meta.page,
			pageSize: meta.limit,
			totalRecords: meta.total_records,
			totalPages: meta.total_pages
		},
		requestId: envelope.request_id
	};
}

export function getOnlineOrder(id: number) {
	return apiGet<OnlineOrderDetail>(`/online-orders/${id}`);
}

export function packOnlineOrder(id: number) {
	return apiPost<OnlineOrderDetail>(`/online-orders/${id}/pack`);
}

export function readyOnlineOrder(id: number) {
	return apiPost<OnlineOrderDetail>(`/online-orders/${id}/ready`);
}

export function deliverOnlineOrder(id: number) {
	return apiPost<OnlineOrderDetail>(`/online-orders/${id}/deliver`);
}

export function completeOnlineOrder(id: number) {
	return apiPost<OnlineOrderDetail>(`/online-orders/${id}/complete`);
}

export function cancelOnlineOrder(id: number, reason?: string | null) {
	return apiPost<OnlineOrderDetail, { reason?: string | null }>(`/online-orders/${id}/cancel`, {
		reason: reason ?? undefined
	});
}

export function markOnlineOrderPaid(id: number) {
	return apiPost<OnlineOrderDetail>(`/online-orders/${id}/mark-paid`);
}

export function getStoreSettings() {
	return apiGet<StoreSettings>('/store-settings');
}

export function updateStoreSettings(payload: UpdateStoreSettingsPayload) {
	return apiPut<StoreSettings, UpdateStoreSettingsPayload>('/store-settings', payload);
}

export function uploadQrisImage(file: File) {
	const formData = new FormData();
	formData.append('file', file);
	return apiPost<UploadResponse>('/store-settings/qris-image', formData);
}

function getFallbackMeta(page: number, limit: number, recordCount: number): PaginationMeta {
	const previousRecords = (page - 1) * limit;
	const hasPossibleNextPage = recordCount === limit;
	const totalRecords = previousRecords + recordCount + (hasPossibleNextPage ? 1 : 0);

	return {
		page,
		limit,
		total_records: totalRecords,
		total_pages: Math.max(0, page + (hasPossibleNextPage ? 1 : 0))
	};
}

function normalizePage(value: number | undefined) {
	if (!Number.isFinite(value)) return 1;
	return Math.max(1, Math.trunc(value ?? 1));
}

function normalizeLimit(value: number | undefined) {
	if (!Number.isFinite(value)) return 10;
	return Math.min(100, Math.max(1, Math.trunc(value ?? 10)));
}

function normalizeStatuses(value: OnlineOrderStatus[] | undefined) {
	if (!value?.length) return [];
	return value.filter(
		(status, index) => onlineOrderStatuses.includes(status) && value.indexOf(status) === index
	);
}
