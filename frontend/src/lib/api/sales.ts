import { apiGet, apiGetEnvelope, apiPost } from './client.js';
import type { PaginationMeta } from './types.js';

export const saleStatuses = ['completed', 'cancelled'] as const;

export type SaleStatus = (typeof saleStatuses)[number];

export type Sale = {
	id: number;
	transaction_no: string;
	sold_at: string;
	cashier_id: number;
	cashier_name: string;
	subtotal_amount: number;
	total_amount: number;
	paid_amount: number;
	change_amount: number;
	status: SaleStatus;
	notes: string | null;
	cancelled_at: string | null;
	cancelled_by: number | null;
	cancel_reason: string | null;
	created_at: string;
	updated_at: string;
};

export type SaleItem = {
	id: number;
	sale_id: number;
	product_id: number;
	product_sku_id: number;
	product_name: string;
	product_sku: string;
	unit: string;
	quantity: number;
	unit_price: number;
	line_total: number;
	created_at: string;
};

export type SaleDetail = Sale & {
	items: SaleItem[];
};

export type SaleListParams = {
	page?: number;
	limit?: number;
	search?: string;
	from?: string;
	to?: string;
	status?: SaleStatus[];
};

export type SalePagination = {
	page: number;
	pageSize: number;
	totalRecords: number;
	totalPages: number;
};

export type CreateSaleItemPayload = {
	product_sku_id: number;
	quantity: number;
};

export type CreateSalePayload = {
	sold_at?: string;
	paid_amount: number;
	notes?: string | null;
	items: CreateSaleItemPayload[];
};

export type CancelSalePayload = {
	reason?: string | null;
};

export type SaleListResult = {
	sales: Sale[];
	pagination: SalePagination;
	requestId: string;
};

export async function listSales(params: SaleListParams = {}): Promise<SaleListResult> {
	const limit = normalizeLimit(params.limit);
	const page = normalizePage(params.page);
	const search = params.search?.trim();
	const from = params.from?.trim();
	const to = params.to?.trim();
	const status = normalizeStatuses(params.status);

	const envelope = await apiGetEnvelope<Sale[], PaginationMeta>('/sales', {
		params: {
			limit,
			offset: (page - 1) * limit,
			search,
			from,
			to,
			status: status.length ? status.join(',') : undefined
		}
	});
	const sales = envelope.data ?? [];
	const meta = envelope.meta ?? getFallbackMeta(page, limit, sales.length);

	return {
		sales,
		pagination: {
			page: meta.page,
			pageSize: meta.limit,
			totalRecords: meta.total_records,
			totalPages: meta.total_pages
		},
		requestId: envelope.request_id
	};
}

export function getSale(id: number) {
	return apiGet<SaleDetail>(`/sales/${id}`);
}

export function createSale(payload: CreateSalePayload) {
	return apiPost<SaleDetail, CreateSalePayload>('/sales', payload);
}

export function cancelSale(id: number, payload: CancelSalePayload) {
	return apiPost<SaleDetail, CancelSalePayload>(`/sales/${id}/cancel`, payload);
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

function normalizeStatuses(value: SaleStatus[] | undefined) {
	if (!value?.length) return [];
	return value.filter(
		(status, index) => saleStatuses.includes(status) && value.indexOf(status) === index
	);
}
