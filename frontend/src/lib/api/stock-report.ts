import { apiGetEnvelope } from './client.js';
import type { PaginationMeta } from './types.js';

export const stockReportStatuses = ['low_stock', 'in_stock', 'inactive'] as const;

export type StockReportStatus = (typeof stockReportStatuses)[number];

export type StockReportItem = {
	product_sku_id: number;
	product_id: number;
	product_name: string;
	sku: string;
	barcode: string | null;
	category_name: string | null;
	purchase_price: number;
	selling_price: number;
	stock: number;
	minimum_stock: number;
	unit: string;
	quantity_on_order: number;
	is_low_stock: boolean;
	is_active: boolean;
};

export type StockReportListParams = {
	page?: number;
	limit?: number;
	search?: string;
	status?: StockReportStatus[];
};

export type StockReportPagination = {
	page: number;
	pageSize: number;
	totalRecords: number;
	totalPages: number;
};

export type StockReportListResult = {
	items: StockReportItem[];
	pagination: StockReportPagination;
	requestId: string;
};

export async function listStockReport(
	params: StockReportListParams = {}
): Promise<StockReportListResult> {
	const limit = normalizeLimit(params.limit);
	const page = normalizePage(params.page);
	const search = params.search?.trim();
	const status = normalizeStatuses(params.status);

	const envelope = await apiGetEnvelope<StockReportItem[], PaginationMeta>('/reports/stock', {
		params: {
			limit,
			offset: (page - 1) * limit,
			search,
			status: status.length ? status.join(',') : undefined
		}
	});
	const items = envelope.data ?? [];
	const meta = envelope.meta ?? getFallbackMeta(page, limit, items.length);

	return {
		items,
		pagination: {
			page: meta.page,
			pageSize: meta.limit,
			totalRecords: meta.total_records,
			totalPages: meta.total_pages
		},
		requestId: envelope.request_id
	};
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

function normalizeStatuses(value: StockReportStatus[] | undefined) {
	if (!value?.length) return [];
	return value.filter(
		(status, index) => stockReportStatuses.includes(status) && value.indexOf(status) === index
	);
}
