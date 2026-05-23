import { apiGet, apiGetEnvelope, apiPost } from './client.js';
import type { PaginationMeta } from './types.js';

export const goodsReceiptStatuses = ['draft', 'posted', 'cancelled'] as const;

export type GoodsReceiptStatus = (typeof goodsReceiptStatuses)[number];

export type GoodsReceipt = {
	id: number;
	gr_no: string;
	purchase_order_id: number;
	po_no: string;
	received_at: string;
	status: GoodsReceiptStatus;
	received_by: number | null;
	posted_at: string | null;
	posted_by: number | null;
	cancelled_at: string | null;
	cancelled_by: number | null;
	cancel_reason: string | null;
	notes: string | null;
	created_at: string;
	updated_at: string;
};

export type GoodsReceiptItem = {
	id: number;
	goods_receipt_id: number;
	purchase_order_item_id: number;
	product_id: number;
	product_sku_id: number;
	product_name: string;
	product_sku: string;
	unit: string;
	quantity_received: number;
	unit_cost: number;
	line_total: number;
	created_at: string;
	updated_at: string;
};

export type GoodsReceiptDetail = GoodsReceipt & {
	items: GoodsReceiptItem[];
};

export type GoodsReceiptListParams = {
	page?: number;
	limit?: number;
	search?: string;
	from?: string;
	to?: string;
	status?: GoodsReceiptStatus[];
};

export type GoodsReceiptPagination = {
	page: number;
	pageSize: number;
	totalRecords: number;
	totalPages: number;
};

export type CreateGoodsReceiptItemPayload = {
	purchase_order_item_id: number;
	quantity_received: number;
	unit_cost?: number | null;
};

export type CreateGoodsReceiptPayload = {
	purchase_order_id: number;
	received_at?: string;
	notes?: string | null;
	items: CreateGoodsReceiptItemPayload[];
};

export type CancelGoodsReceiptPayload = {
	reason?: string | null;
};

export type GoodsReceiptListResult = {
	goodsReceipts: GoodsReceipt[];
	pagination: GoodsReceiptPagination;
	requestId: string;
};

export async function listGoodsReceipts(
	params: GoodsReceiptListParams = {}
): Promise<GoodsReceiptListResult> {
	const limit = normalizeLimit(params.limit);
	const page = normalizePage(params.page);
	const search = params.search?.trim();
	const from = params.from?.trim();
	const to = params.to?.trim();
	const status = normalizeStatuses(params.status);

	const envelope = await apiGetEnvelope<GoodsReceipt[], PaginationMeta>('/goods-receipts', {
		params: {
			limit,
			offset: (page - 1) * limit,
			search,
			from,
			to,
			status: status.length ? status.join(',') : undefined
		}
	});
	const goodsReceipts = envelope.data ?? [];
	const meta = envelope.meta ?? getFallbackMeta(page, limit, goodsReceipts.length);

	return {
		goodsReceipts,
		pagination: {
			page: meta.page,
			pageSize: meta.limit,
			totalRecords: meta.total_records,
			totalPages: meta.total_pages
		},
		requestId: envelope.request_id
	};
}

export function getGoodsReceipt(id: number) {
	return apiGet<GoodsReceiptDetail>(`/goods-receipts/${id}`);
}

export function createGoodsReceipt(payload: CreateGoodsReceiptPayload) {
	return apiPost<GoodsReceiptDetail, CreateGoodsReceiptPayload>('/goods-receipts', payload);
}

export function postGoodsReceipt(id: number) {
	return apiPost<GoodsReceiptDetail>(`/goods-receipts/${id}/post`);
}

export function cancelGoodsReceipt(id: number, payload: CancelGoodsReceiptPayload) {
	return apiPost<GoodsReceiptDetail, CancelGoodsReceiptPayload>(
		`/goods-receipts/${id}/cancel`,
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

function normalizeStatuses(value: GoodsReceiptStatus[] | undefined) {
	if (!value?.length) return [];
	return value.filter(
		(status, index) => goodsReceiptStatuses.includes(status) && value.indexOf(status) === index
	);
}
