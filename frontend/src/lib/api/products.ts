import { apiDelete, apiGet, apiGetEnvelope, apiPost, apiPut } from './client.js';
import type { PaginationMeta } from './types.js';

export type Product = {
	id: number;
	category_id: number | null;
	category_name: string | null;
	name: string;
	is_active: boolean;
	created_at: string;
	updated_at: string;
	deleted_at: string | null;
};

export type ProductListParams = {
	page?: number;
	limit?: number;
	search?: string;
};

export type ProductPagination = {
	page: number;
	pageSize: number;
	totalRecords: number;
	totalPages: number;
};

export type CreateProductPayload = {
	category_id?: number | null;
	name: string;
	is_active?: boolean;
};

export type UpdateProductPayload = {
	category_id?: number | null;
	name?: string;
	is_active?: boolean;
};

export type MessageResponse = {
	message: string;
};

export type ProductListResult = {
	products: Product[];
	pagination: ProductPagination;
	requestId: string;
};

export async function listProducts(params: ProductListParams = {}): Promise<ProductListResult> {
	const limit = normalizeLimit(params.limit);
	const page = normalizePage(params.page);
	const search = params.search?.trim();

	const envelope = await apiGetEnvelope<Product[], PaginationMeta>('/products', {
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
		products: envelope.data ?? [],
		pagination: {
			page: meta.page,
			pageSize: meta.limit,
			totalRecords: meta.total_records,
			totalPages: meta.total_pages
		},
		requestId: envelope.request_id
	};
}

export function getProduct(id: number) {
	return apiGet<Product>(`/products/${id}`);
}

export function createProduct(payload: CreateProductPayload) {
	return apiPost<Product, CreateProductPayload>('/products', payload);
}

export function updateProduct(id: number, payload: UpdateProductPayload) {
	return apiPut<Product, UpdateProductPayload>(`/products/${id}`, payload);
}

export function deleteProduct(id: number) {
	return apiDelete<MessageResponse>(`/products/${id}`);
}

function normalizePage(value: number | undefined) {
	if (!Number.isFinite(value)) return 1;
	return Math.max(1, Math.trunc(value ?? 1));
}

function normalizeLimit(value: number | undefined) {
	if (!Number.isFinite(value)) return 10;
	return Math.min(100, Math.max(1, Math.trunc(value ?? 10)));
}
