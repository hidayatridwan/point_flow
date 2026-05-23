import { apiGet, apiGetEnvelope, apiPost } from './client.js';
import type { PaginationMeta } from './types.js';

export const purchaseOrderStatuses = [
	'draft',
	'ordered',
	'partially_received',
	'received',
	'cancelled'
] as const;

export type PurchaseOrderStatus = (typeof purchaseOrderStatuses)[number];

export type PurchaseOrder = {
	id: number;
	po_no: string;
	supplier_id: number;
	supplier_name: string;
	order_date: string;
	expected_date: string | null;
	status: PurchaseOrderStatus;
	subtotal_amount: number;
	notes: string | null;
	ordered_by: number | null;
	ordered_at: string | null;
	cancelled_at: string | null;
	cancelled_by: number | null;
	cancel_reason: string | null;
	created_by: number | null;
	created_at: string;
	updated_at: string;
};

export type PurchaseOrderItem = {
	id: number;
	purchase_order_id: number;
	product_id: number;
	product_sku_id: number;
	product_name: string;
	product_sku: string;
	unit: string;
	quantity_ordered: number;
	quantity_received: number;
	unit_cost: number;
	line_total: number;
	created_at: string;
	updated_at: string;
};

export type PurchaseOrderDetail = PurchaseOrder & {
	items: PurchaseOrderItem[];
};

export type PurchaseOrderListParams = {
	page?: number;
	limit?: number;
	search?: string;
	from?: string;
	to?: string;
	status?: PurchaseOrderStatus[];
};

export type PurchaseOrderPagination = {
	page: number;
	pageSize: number;
	totalRecords: number;
	totalPages: number;
};

export type CreatePurchaseOrderItemPayload = {
	product_sku_id: number;
	quantity_ordered: number;
	unit_cost: number;
};

export type CreatePurchaseOrderPayload = {
	supplier_id: number;
	order_date?: string;
	expected_date?: string | null;
	notes?: string | null;
	items: CreatePurchaseOrderItemPayload[];
};

export type CancelPurchaseOrderPayload = {
	reason?: string | null;
};

export type PurchaseOrderListResult = {
	purchaseOrders: PurchaseOrder[];
	pagination: PurchaseOrderPagination;
	requestId: string;
};

export async function listPurchaseOrders(
	params: PurchaseOrderListParams = {}
): Promise<PurchaseOrderListResult> {
	const limit = normalizeLimit(params.limit);
	const page = normalizePage(params.page);
	const search = params.search?.trim();
	const from = params.from?.trim();
	const to = params.to?.trim();
	const status = normalizeStatuses(params.status);

	const envelope = await apiGetEnvelope<PurchaseOrder[], PaginationMeta>('/purchase-orders', {
		params: {
			limit,
			offset: (page - 1) * limit,
			search,
			from,
			to,
			status: status.length ? status.join(',') : undefined
		}
	});
	const purchaseOrders = envelope.data ?? [];
	const meta = envelope.meta ?? getFallbackMeta(page, limit, purchaseOrders.length);

	return {
		purchaseOrders,
		pagination: {
			page: meta.page,
			pageSize: meta.limit,
			totalRecords: meta.total_records,
			totalPages: meta.total_pages
		},
		requestId: envelope.request_id
	};
}

export function getPurchaseOrder(id: number) {
	return apiGet<PurchaseOrderDetail>(`/purchase-orders/${id}`);
}

export function createPurchaseOrder(payload: CreatePurchaseOrderPayload) {
	return apiPost<PurchaseOrderDetail, CreatePurchaseOrderPayload>('/purchase-orders', payload);
}

export function orderPurchaseOrder(id: number) {
	return apiPost<PurchaseOrderDetail>(`/purchase-orders/${id}/order`);
}

export function cancelPurchaseOrder(id: number, payload: CancelPurchaseOrderPayload) {
	return apiPost<PurchaseOrderDetail, CancelPurchaseOrderPayload>(
		`/purchase-orders/${id}/cancel`,
		payload
	);
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

function normalizeStatuses(value: PurchaseOrderStatus[] | undefined) {
	if (!value?.length) return [];
	return value.filter(
		(status, index) => purchaseOrderStatuses.includes(status) && value.indexOf(status) === index
	);
}
