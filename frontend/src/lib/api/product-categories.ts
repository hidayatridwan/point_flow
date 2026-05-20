import { apiDelete, apiGet, apiGetEnvelope, apiPost, apiPut } from './client.js';
import type { PaginationMeta } from './types.js';

export type ProductCategory = {
	id: number;
	name: string;
	description: string | null;
	is_active: boolean;
	created_at: string;
	updated_at: string;
	deleted_at: string | null;
};

export type ProductCategoryListParams = {
	page?: number;
	limit?: number;
	search?: string;
};

export type ProductCategoryPagination = {
	page: number;
	pageSize: number;
	totalRecords: number;
	totalPages: number;
};

export type UpdateProductCategoryPayload = {
	name?: string;
	description?: string | null;
	is_active?: boolean;
};

export type CreateProductCategoryPayload = {
	name: string;
	description?: string | null;
	is_active?: boolean;
};

export type MessageResponse = {
	message: string;
};

export type ProductCategoryListResult = {
	categories: ProductCategory[];
	pagination: ProductCategoryPagination;
	requestId: string;
};

export async function listProductCategories(
	params: ProductCategoryListParams = {}
): Promise<ProductCategoryListResult> {
	const limit = normalizeLimit(params.limit);
	const page = normalizePage(params.page);
	const search = params.search?.trim();

	const envelope = await apiGetEnvelope<ProductCategory[], PaginationMeta>('/product-categories', {
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
		categories: envelope.data ?? [],
		pagination: {
			page: meta.page,
			pageSize: meta.limit,
			totalRecords: meta.total_records,
			totalPages: meta.total_pages
		},
		requestId: envelope.request_id
	};
}

export function getProductCategory(id: number) {
	return apiGet<ProductCategory>(`/product-categories/${id}`);
}

export function createProductCategory(payload: CreateProductCategoryPayload) {
	return apiPost<ProductCategory, CreateProductCategoryPayload>('/product-categories', payload);
}

export function updateProductCategory(id: number, payload: UpdateProductCategoryPayload) {
	return apiPut<ProductCategory, UpdateProductCategoryPayload>(`/product-categories/${id}`, payload);
}

export function deleteProductCategory(id: number) {
	return apiDelete<MessageResponse>(`/product-categories/${id}`);
}

function normalizePage(value: number | undefined) {
	if (!Number.isFinite(value)) return 1;
	return Math.max(1, Math.trunc(value ?? 1));
}

function normalizeLimit(value: number | undefined) {
	if (!Number.isFinite(value)) return 10;
	return Math.min(100, Math.max(1, Math.trunc(value ?? 10)));
}
