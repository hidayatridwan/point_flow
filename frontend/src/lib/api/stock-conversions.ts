import { apiGet, apiGetEnvelope, apiPost } from './client.js';
import type { PaginationMeta } from './types.js';

export const stockConversionStatuses = ['draft', 'posted', 'cancelled'] as const;

export type StockConversionStatus = (typeof stockConversionStatuses)[number];

export type StockConversion = {
	id: number;
	conversion_no: string;
	converted_at: string;
	status: StockConversionStatus;
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

export type StockConversionItem = {
	id: number;
	stock_conversion_id: number;
	from_product_id: number;
	from_product_sku_id: number;
	from_product_name: string;
	from_product_sku: string;
	from_unit: string;
	from_stock_before: number;
	from_quantity: number;
	from_stock_after: number;
	to_product_id: number;
	to_product_sku_id: number;
	to_product_name: string;
	to_product_sku: string;
	to_unit: string;
	to_stock_before: number;
	to_quantity: number;
	to_stock_after: number;
	notes: string | null;
	created_at: string;
	updated_at: string;
};

export type StockConversionDetail = StockConversion & {
	items: StockConversionItem[];
};

export type StockConversionListParams = {
	page?: number;
	limit?: number;
	search?: string;
	from?: string;
	to?: string;
	status?: StockConversionStatus[];
};

export type StockConversionPagination = {
	page: number;
	pageSize: number;
	totalRecords: number;
	totalPages: number;
};

export type CreateStockConversionItemPayload = {
	from_product_sku_id: number;
	from_quantity: number;
	to_product_sku_id: number;
	to_quantity: number;
	notes?: string | null;
};

export type CreateStockConversionPayload = {
	converted_at?: string;
	notes?: string | null;
	items: CreateStockConversionItemPayload[];
};

export type CancelStockConversionPayload = {
	reason?: string | null;
};

export type StockConversionListResult = {
	stockConversions: StockConversion[];
	pagination: StockConversionPagination;
	requestId: string;
};

export async function listStockConversions(
	params: StockConversionListParams = {}
): Promise<StockConversionListResult> {
	const limit = normalizeLimit(params.limit);
	const page = normalizePage(params.page);
	const search = params.search?.trim();
	const from = params.from?.trim();
	const to = params.to?.trim();
	const status = normalizeStatuses(params.status);

	const envelope = await apiGetEnvelope<StockConversion[], PaginationMeta>('/stock-conversions', {
		params: {
			limit,
			offset: (page - 1) * limit,
			search,
			from,
			to,
			status: status.length ? status.join(',') : undefined
		}
	});
	const stockConversions = envelope.data ?? [];
	const meta = envelope.meta ?? getFallbackMeta(page, limit, stockConversions.length);

	return {
		stockConversions,
		pagination: {
			page: meta.page,
			pageSize: meta.limit,
			totalRecords: meta.total_records,
			totalPages: meta.total_pages
		},
		requestId: envelope.request_id
	};
}

export function getStockConversion(id: number) {
	return apiGet<StockConversionDetail>(`/stock-conversions/${id}`);
}

export function createStockConversion(payload: CreateStockConversionPayload) {
	return apiPost<StockConversionDetail, CreateStockConversionPayload>(
		'/stock-conversions',
		payload
	);
}

export function postStockConversion(id: number) {
	return apiPost<StockConversionDetail>(`/stock-conversions/${id}/post`);
}

export function cancelStockConversion(id: number, payload: CancelStockConversionPayload) {
	return apiPost<StockConversionDetail, CancelStockConversionPayload>(
		`/stock-conversions/${id}/cancel`,
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

function normalizeStatuses(value: StockConversionStatus[] | undefined) {
	if (!value?.length) return [];
	return value.filter(
		(status, index) => stockConversionStatuses.includes(status) && value.indexOf(status) === index
	);
}
