import { apiDelete, apiGet, apiGetEnvelope, apiPost, apiPut } from './client.js';
import type { PaginationMeta } from './types.js';

export type ProductSku = {
	id: number;
	product_id: number;
	product_name: string;
	category_id: number | null;
	category_name: string | null;
	sku: string;
	barcode: string | null;
	unit_id: number;
	unit_code: string;
	unit_name: string;
	contents_qty: number;
	purchase_price: number;
	selling_price: number;
	stock: number;
	minimum_stock: number;
	is_active: boolean;
	created_at: string;
	updated_at: string;
	deleted_at: string | null;
};

export type ProductSkuListParams = {
	page?: number;
	limit?: number;
	search?: string;
};

export type ProductSkuPagination = {
	page: number;
	pageSize: number;
	totalRecords: number;
	totalPages: number;
};

export type CreateProductSkuPayload = {
	product_id: number;
	sku: string;
	barcode?: string | null;
	unit_id: number;
	contents_qty?: number;
	purchase_price: number;
	selling_price: number;
	stock?: number;
	minimum_stock?: number;
	is_active?: boolean;
};

export type UpdateProductSkuPayload = {
	sku?: string;
	barcode?: string | null;
	unit_id?: number;
	contents_qty?: number;
	purchase_price?: number;
	selling_price?: number;
	stock?: number;
	minimum_stock?: number;
	is_active?: boolean;
};

export type MessageResponse = {
	message: string;
};

export type ProductSkuListResult = {
	productSkus: ProductSku[];
	pagination: ProductSkuPagination;
	requestId: string;
};

export async function listProductSkus(
	params: ProductSkuListParams = {}
): Promise<ProductSkuListResult> {
	const limit = normalizeLimit(params.limit);
	const page = normalizePage(params.page);
	const search = params.search?.trim();

	const envelope = await apiGetEnvelope<ProductSku[], PaginationMeta>('/product-skus', {
		params: {
			limit,
			offset: (page - 1) * limit,
			search
		}
	});
	const meta = envelope.meta ?? {
		page,
		limit,
		total_records: envelope.data?.length ?? 0,
		total_pages: envelope.data?.length ? 1 : 0
	};

	return {
		productSkus: envelope.data ?? [],
		pagination: {
			page: meta.page,
			pageSize: meta.limit,
			totalRecords: meta.total_records,
			totalPages: meta.total_pages
		},
		requestId: envelope.request_id
	};
}

export function getProductSku(id: number) {
	return apiGet<ProductSku>(`/product-skus/${id}`);
}

export function createProductSku(payload: CreateProductSkuPayload) {
	return apiPost<ProductSku, CreateProductSkuPayload>('/product-skus', payload);
}

export function updateProductSku(id: number, payload: UpdateProductSkuPayload) {
	return apiPut<ProductSku, UpdateProductSkuPayload>(`/product-skus/${id}`, payload);
}

export function deleteProductSku(id: number) {
	return apiDelete<MessageResponse>(`/product-skus/${id}`);
}

function normalizePage(value: number | undefined) {
	if (!Number.isFinite(value)) return 1;
	return Math.max(1, Math.trunc(value ?? 1));
}

function normalizeLimit(value: number | undefined) {
	if (!Number.isFinite(value)) return 10;
	return Math.min(100, Math.max(1, Math.trunc(value ?? 10)));
}
