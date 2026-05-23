import { apiGetEnvelope } from './client.js';
import type { PaginationMeta } from './types.js';

export const stockMovementTypes = [
	'stock_in',
	'stock_out',
	'sale',
	'purchase_receipt',
	'adjustment',
	'conversion_in',
	'conversion_out',
	'correction'
] as const;

export type StockMovementType = (typeof stockMovementTypes)[number];

export type StockMovementDirection = 'in' | 'out';

export type StockMovement = {
	id: number;
	created_at: string;
	product_id: number;
	product_sku_id: number;
	product_name: string;
	sku: string;
	unit: string;
	movement_type: StockMovementType;
	direction: StockMovementDirection;
	quantity: number;
	stock_before: number;
	stock_after: number;
	quantity_delta: number;
	reference_type: string;
	reference_id: number | null;
	reference_no: string | null;
	notes: string | null;
	created_by: number | null;
};

export type StockMovementListParams = {
	page?: number;
	limit?: number;
	search?: string;
	from?: string;
	to?: string;
	movementType?: StockMovementType[];
	productSkuId?: number | null;
};

export type StockMovementPagination = {
	page: number;
	pageSize: number;
	totalRecords: number;
	totalPages: number;
};

export type StockMovementListResult = {
	stockMovements: StockMovement[];
	pagination: StockMovementPagination;
	requestId: string;
};

export async function listStockMovements(
	params: StockMovementListParams = {}
): Promise<StockMovementListResult> {
	const limit = normalizeLimit(params.limit);
	const page = normalizePage(params.page);
	const search = params.search?.trim();
	const from = params.from?.trim();
	const to = params.to?.trim();
	const movementType = normalizeMovementTypes(params.movementType);

	const envelope = await apiGetEnvelope<StockMovement[], PaginationMeta>(
		'/reports/stock-movements',
		{
			params: {
				limit,
				offset: (page - 1) * limit,
				search,
				from,
				to,
				type: movementType.length ? movementType.join(',') : undefined,
				product_sku_id: params.productSkuId ?? undefined
			}
		}
	);
	const stockMovements = envelope.data ?? [];
	const meta = envelope.meta ?? getFallbackMeta(page, limit, stockMovements.length);

	return {
		stockMovements,
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

function normalizeMovementTypes(value: StockMovementType[] | undefined) {
	if (!value?.length) return [];
	return value.filter(
		(movementType, index) =>
			stockMovementTypes.includes(movementType) && value.indexOf(movementType) === index
	);
}
