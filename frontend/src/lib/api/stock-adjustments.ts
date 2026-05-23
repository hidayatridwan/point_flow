import { apiGet, apiGetEnvelope, apiPost } from './client.js';
import type { PaginationMeta } from './types.js';

export const stockAdjustmentStatuses = ['draft', 'posted', 'cancelled'] as const;

export type StockAdjustmentStatus = (typeof stockAdjustmentStatuses)[number];

export type StockAdjustment = {
	id: number;
	adjustment_no: string;
	adjusted_at: string;
	status: StockAdjustmentStatus;
	reason: string;
	notes: string | null;
	created_by: number | null;
	posted_at: string | null;
	posted_by: number | null;
	cancelled_at: string | null;
	cancelled_by: number | null;
	cancel_reason: string | null;
	created_at: string;
	updated_at: string;
};

export type StockAdjustmentItem = {
	id: number;
	stock_adjustment_id: number;
	product_id: number;
	product_sku_id: number;
	product_name: string;
	product_sku: string;
	unit: string;
	stock_before: number;
	stock_after: number;
	quantity_delta: number;
	notes: string | null;
	created_at: string;
	updated_at: string;
};

export type StockAdjustmentDetail = StockAdjustment & {
	items: StockAdjustmentItem[];
};

export type StockAdjustmentListParams = {
	page?: number;
	limit?: number;
	search?: string;
	from?: string;
	to?: string;
	status?: StockAdjustmentStatus[];
};

export type StockAdjustmentPagination = {
	page: number;
	pageSize: number;
	totalRecords: number;
	totalPages: number;
};

export type CreateStockAdjustmentItemPayload = {
	product_sku_id: number;
	stock_after: number;
	notes?: string | null;
};

export type CreateStockAdjustmentPayload = {
	adjusted_at?: string;
	reason: string;
	notes?: string | null;
	items: CreateStockAdjustmentItemPayload[];
};

export type CancelStockAdjustmentPayload = {
	reason?: string | null;
};

export type StockAdjustmentListResult = {
	stockAdjustments: StockAdjustment[];
	pagination: StockAdjustmentPagination;
	requestId: string;
};

export async function listStockAdjustments(
	params: StockAdjustmentListParams = {}
): Promise<StockAdjustmentListResult> {
	const limit = normalizeLimit(params.limit);
	const page = normalizePage(params.page);
	const search = params.search?.trim();
	const from = params.from?.trim();
	const to = params.to?.trim();
	const status = normalizeStatuses(params.status);

	const envelope = await apiGetEnvelope<StockAdjustment[], PaginationMeta>('/stock-adjustments', {
		params: {
			limit,
			offset: (page - 1) * limit,
			search,
			from,
			to,
			status: status.length ? status.join(',') : undefined
		}
	});
	const stockAdjustments = envelope.data ?? [];
	const meta = envelope.meta ?? getFallbackMeta(page, limit, stockAdjustments.length);

	return {
		stockAdjustments,
		pagination: {
			page: meta.page,
			pageSize: meta.limit,
			totalRecords: meta.total_records,
			totalPages: meta.total_pages
		},
		requestId: envelope.request_id
	};
}

export function getStockAdjustment(id: number) {
	return apiGet<StockAdjustmentDetail>(`/stock-adjustments/${id}`);
}

export function createStockAdjustment(payload: CreateStockAdjustmentPayload) {
	return apiPost<StockAdjustmentDetail, CreateStockAdjustmentPayload>(
		'/stock-adjustments',
		payload
	);
}

export function postStockAdjustment(id: number) {
	return apiPost<StockAdjustmentDetail>(`/stock-adjustments/${id}/post`);
}

export function cancelStockAdjustment(id: number, payload: CancelStockAdjustmentPayload) {
	return apiPost<StockAdjustmentDetail, CancelStockAdjustmentPayload>(
		`/stock-adjustments/${id}/cancel`,
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

function normalizeStatuses(value: StockAdjustmentStatus[] | undefined) {
	if (!value?.length) return [];
	return value.filter(
		(status, index) => stockAdjustmentStatuses.includes(status) && value.indexOf(status) === index
	);
}
