import { apiDelete, apiGet, apiGetEnvelope, apiPost, apiPut } from './client.js';
import type { PaginationMeta } from './types.js';

export type Supplier = {
	id: number;
	name: string;
	contact_name: string | null;
	phone: string | null;
	email: string | null;
	address: string | null;
	is_active: boolean;
	created_at: string;
	updated_at: string;
	deleted_at: string | null;
};

export type SupplierListParams = {
	page?: number;
	limit?: number;
	search?: string;
};

export type SupplierPagination = {
	page: number;
	pageSize: number;
	totalRecords: number;
	totalPages: number;
};

export type CreateSupplierPayload = {
	name: string;
	contact_name?: string | null;
	phone?: string | null;
	email?: string | null;
	address?: string | null;
	is_active?: boolean;
};

export type UpdateSupplierPayload = {
	name?: string;
	contact_name?: string | null;
	phone?: string | null;
	email?: string | null;
	address?: string | null;
	is_active?: boolean;
};

export type MessageResponse = {
	message: string;
};

export type SupplierListResult = {
	suppliers: Supplier[];
	pagination: SupplierPagination;
	requestId: string;
};

export async function listSuppliers(params: SupplierListParams = {}): Promise<SupplierListResult> {
	const limit = normalizeLimit(params.limit);
	const page = normalizePage(params.page);
	const search = params.search?.trim();

	const envelope = await apiGetEnvelope<Supplier[], PaginationMeta>('/suppliers', {
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
		suppliers: envelope.data ?? [],
		pagination: {
			page: meta.page,
			pageSize: meta.limit,
			totalRecords: meta.total_records,
			totalPages: meta.total_pages
		},
		requestId: envelope.request_id
	};
}

export function getSupplier(id: number) {
	return apiGet<Supplier>(`/suppliers/${id}`);
}

export function createSupplier(payload: CreateSupplierPayload) {
	return apiPost<Supplier, CreateSupplierPayload>('/suppliers', payload);
}

export function updateSupplier(id: number, payload: UpdateSupplierPayload) {
	return apiPut<Supplier, UpdateSupplierPayload>(`/suppliers/${id}`, payload);
}

export function deleteSupplier(id: number) {
	return apiDelete<MessageResponse>(`/suppliers/${id}`);
}

function normalizePage(value: number | undefined) {
	if (!Number.isFinite(value)) return 1;
	return Math.max(1, Math.trunc(value ?? 1));
}

function normalizeLimit(value: number | undefined) {
	if (!Number.isFinite(value)) return 10;
	return Math.min(100, Math.max(1, Math.trunc(value ?? 10)));
}
