import { apiGet, apiGetEnvelope } from './client.js';
import type { GoodsReceiptStatus } from './goods-receipts.js';
import { goodsReceiptStatuses } from './goods-receipts.js';
import type { PurchaseOrderStatus } from './purchase-orders.js';
import { purchaseOrderStatuses } from './purchase-orders.js';
import type { StockAdjustmentStatus } from './stock-adjustments.js';
import { stockAdjustmentStatuses } from './stock-adjustments.js';
import type { PaginationMeta } from './types.js';

export type ReportPagination = {
	page: number;
	pageSize: number;
	totalRecords: number;
	totalPages: number;
};

export type SalesReportListParams = {
	page?: number;
	limit?: number;
	from?: string;
	to?: string;
};

export type DocumentReportListParams<TStatus extends string> = {
	page?: number;
	limit?: number;
	search?: string;
	from?: string;
	to?: string;
	status?: TStatus[];
};

export type DailySalesReport = {
	sales_date: string;
	transaction_count: number;
	subtotal_amount: number;
	total_amount: number;
	cash_revenue: number;
	noncash_revenue: number;
};

export type MonthlySalesReport = {
	sales_month: string;
	transaction_count: number;
	subtotal_amount: number;
	total_amount: number;
	cash_revenue: number;
	noncash_revenue: number;
};

export type PurchaseOrderReport = {
	id: number;
	po_no: string;
	order_date: string;
	expected_date: string | null;
	supplier_name: string;
	status: PurchaseOrderStatus;
	item_count: number;
	total_quantity_ordered: number;
	total_quantity_received: number;
	total_quantity_remaining: number;
	subtotal_amount: number;
	created_at: string;
	updated_at: string;
};

export type GoodsReceiptReport = {
	id: number;
	gr_no: string;
	received_at: string;
	status: GoodsReceiptStatus;
	po_no: string;
	supplier_name: string;
	item_count: number;
	total_quantity_received: number;
	total_amount: number;
	created_at: string;
	updated_at: string;
};

export type StockAdjustmentReport = {
	id: number;
	adjustment_no: string;
	adjusted_at: string;
	status: StockAdjustmentStatus;
	reason: string;
	item_count: number;
	total_quantity_increased: number;
	total_quantity_decreased: number;
	created_by: number | null;
	posted_at: string | null;
	posted_by: number | null;
	created_at: string;
	updated_at: string;
};

export type DailySalesReportListResult = {
	reports: DailySalesReport[];
	pagination: ReportPagination;
	requestId: string;
};

export type MonthlySalesReportListResult = {
	reports: MonthlySalesReport[];
	pagination: ReportPagination;
	requestId: string;
};

export type PurchaseOrderReportListParams = DocumentReportListParams<PurchaseOrderStatus>;
export type PurchaseOrderReportListResult = {
	reports: PurchaseOrderReport[];
	pagination: ReportPagination;
	requestId: string;
};

export type GoodsReceiptReportListParams = DocumentReportListParams<GoodsReceiptStatus>;
export type GoodsReceiptReportListResult = {
	reports: GoodsReceiptReport[];
	pagination: ReportPagination;
	requestId: string;
};

export type StockAdjustmentReportListParams = DocumentReportListParams<StockAdjustmentStatus>;
export type StockAdjustmentReportListResult = {
	reports: StockAdjustmentReport[];
	pagination: ReportPagination;
	requestId: string;
};

export async function listDailySalesReport(
	params: SalesReportListParams = {}
): Promise<DailySalesReportListResult> {
	const limit = normalizeLimit(params.limit);
	const page = normalizePage(params.page);
	const from = params.from?.trim();
	const to = params.to?.trim();

	const envelope = await apiGetEnvelope<DailySalesReport[], PaginationMeta>(
		'/reports/daily-sales',
		{
			params: {
				limit,
				offset: (page - 1) * limit,
				from,
				to
			}
		}
	);
	const reports = envelope.data ?? [];
	const meta = envelope.meta ?? getFallbackMeta(page, limit, reports.length);

	return toReportListResult(reports, meta, envelope.request_id);
}

export async function listMonthlySalesReport(
	params: SalesReportListParams = {}
): Promise<MonthlySalesReportListResult> {
	const limit = normalizeLimit(params.limit);
	const page = normalizePage(params.page);
	const from = params.from?.trim();
	const to = params.to?.trim();

	const envelope = await apiGetEnvelope<MonthlySalesReport[], PaginationMeta>(
		'/reports/monthly-sales',
		{
			params: {
				limit,
				offset: (page - 1) * limit,
				from,
				to
			}
		}
	);
	const reports = envelope.data ?? [];
	const meta = envelope.meta ?? getFallbackMeta(page, limit, reports.length);

	return toReportListResult(reports, meta, envelope.request_id);
}

export async function listPurchaseOrderReport(
	params: PurchaseOrderReportListParams = {}
): Promise<PurchaseOrderReportListResult> {
	const limit = normalizeLimit(params.limit);
	const page = normalizePage(params.page);
	const search = params.search?.trim();
	const from = params.from?.trim();
	const to = params.to?.trim();
	const status = normalizeStatuses(params.status, purchaseOrderStatuses);

	const envelope = await apiGetEnvelope<PurchaseOrderReport[], PaginationMeta>(
		'/reports/purchase-orders',
		{
			params: {
				limit,
				offset: (page - 1) * limit,
				search,
				from,
				to,
				status: status.length ? status.join(',') : undefined
			}
		}
	);
	const reports = envelope.data ?? [];
	const meta = envelope.meta ?? getFallbackMeta(page, limit, reports.length);

	return toReportListResult(reports, meta, envelope.request_id);
}

export async function listGoodsReceiptReport(
	params: GoodsReceiptReportListParams = {}
): Promise<GoodsReceiptReportListResult> {
	const limit = normalizeLimit(params.limit);
	const page = normalizePage(params.page);
	const search = params.search?.trim();
	const from = params.from?.trim();
	const to = params.to?.trim();
	const status = normalizeStatuses(params.status, goodsReceiptStatuses);

	const envelope = await apiGetEnvelope<GoodsReceiptReport[], PaginationMeta>(
		'/reports/goods-receipts',
		{
			params: {
				limit,
				offset: (page - 1) * limit,
				search,
				from,
				to,
				status: status.length ? status.join(',') : undefined
			}
		}
	);
	const reports = envelope.data ?? [];
	const meta = envelope.meta ?? getFallbackMeta(page, limit, reports.length);

	return toReportListResult(reports, meta, envelope.request_id);
}

export async function listStockAdjustmentReport(
	params: StockAdjustmentReportListParams = {}
): Promise<StockAdjustmentReportListResult> {
	const limit = normalizeLimit(params.limit);
	const page = normalizePage(params.page);
	const search = params.search?.trim();
	const from = params.from?.trim();
	const to = params.to?.trim();
	const status = normalizeStatuses(params.status, stockAdjustmentStatuses);

	const envelope = await apiGetEnvelope<StockAdjustmentReport[], PaginationMeta>(
		'/reports/stock-adjustments',
		{
			params: {
				limit,
				offset: (page - 1) * limit,
				search,
				from,
				to,
				status: status.length ? status.join(',') : undefined
			}
		}
	);
	const reports = envelope.data ?? [];
	const meta = envelope.meta ?? getFallbackMeta(page, limit, reports.length);

	return toReportListResult(reports, meta, envelope.request_id);
}

function toReportListResult<TReport>(reports: TReport[], meta: PaginationMeta, requestId: string) {
	return {
		reports,
		pagination: {
			page: meta.page,
			pageSize: meta.limit,
			totalRecords: meta.total_records,
			totalPages: meta.total_pages
		},
		requestId
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

function normalizeStatuses<TStatus extends string>(
	value: TStatus[] | undefined,
	allowedStatuses: readonly TStatus[]
) {
	if (!value?.length) return [];
	return value.filter(
		(status, index) => allowedStatuses.includes(status) && value.indexOf(status) === index
	);
}

export type IncomeStatement = {
	gross_revenue: number;
	return_amount: number;
	net_revenue: number;
	cogs: number;
	gross_profit: number;
	other_income: number;
	payroll_expense: number;
	other_expense: number;
	net_profit: number;
};

export type CashFlow = {
	cash_in_sales: number;
	cash_in_manual: number;
	total_cash_in: number;
	cash_out_drawer: number;
	cash_out_manual: number;
	total_cash_out: number;
	net_cash_flow: number;
};

export type FinancialPosition = {
	as_of: string;
	cash: number;
	inventory_value: number;
	total_assets: number;
	cumulative_net_profit: number;
};

export type FinancialPeriodParams = {
	date_from?: string;
	date_to?: string;
};

export type FinancialPositionParams = {
	as_of?: string;
};

export async function getIncomeStatement(
	params: FinancialPeriodParams = {}
): Promise<IncomeStatement> {
	return apiGet<IncomeStatement>('/reports/income-statement', {
		params: {
			date_from: params.date_from?.trim() || undefined,
			date_to: params.date_to?.trim() || undefined
		}
	});
}

export async function getCashFlow(params: FinancialPeriodParams = {}): Promise<CashFlow> {
	return apiGet<CashFlow>('/reports/cash-flow', {
		params: {
			date_from: params.date_from?.trim() || undefined,
			date_to: params.date_to?.trim() || undefined
		}
	});
}

export async function getFinancialPosition(
	params: FinancialPositionParams = {}
): Promise<FinancialPosition> {
	return apiGet<FinancialPosition>('/reports/financial-position', {
		params: {
			as_of: params.as_of?.trim() || undefined
		}
	});
}
