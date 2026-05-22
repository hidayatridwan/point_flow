import { apiDelete, apiGet, apiGetEnvelope, apiPost, apiPut } from './client.js';
import type { PaginationMeta } from './types.js';

export type Unit = {
	id: number;
	code: string;
	name: string;
	allow_decimal: boolean;
	is_active: boolean;
	created_at: string;
	updated_at: string;
	deleted_at: string | null;
};

export type UnitListParams = {
	page?: number;
	limit?: number;
	search?: string;
};

export type UnitPagination = {
	page: number;
	pageSize: number;
	totalRecords: number;
	totalPages: number;
};

export type CreateUnitPayload = {
	code: string;
	name: string;
	allow_decimal?: boolean;
	is_active?: boolean;
};

export type UpdateUnitPayload = {
	code?: string;
	name?: string;
	allow_decimal?: boolean;
	is_active?: boolean;
};

export type MessageResponse = {
	message: string;
};

export type UnitListResult = {
	units: Unit[];
	pagination: UnitPagination;
	requestId: string;
};

export async function listUnits(params: UnitListParams = {}): Promise<UnitListResult> {
	const limit = normalizeLimit(params.limit);
	const page = normalizePage(params.page);
	const search = params.search?.trim();

	const envelope = await apiGetEnvelope<Unit[], PaginationMeta>('/units', {
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
		units: envelope.data ?? [],
		pagination: {
			page: meta.page,
			pageSize: meta.limit,
			totalRecords: meta.total_records,
			totalPages: meta.total_pages
		},
		requestId: envelope.request_id
	};
}

export function getUnit(id: number) {
	return apiGet<Unit>(`/units/${id}`);
}

export function createUnit(payload: CreateUnitPayload) {
	return apiPost<Unit, CreateUnitPayload>('/units', payload);
}

export function updateUnit(id: number, payload: UpdateUnitPayload) {
	return apiPut<Unit, UpdateUnitPayload>(`/units/${id}`, payload);
}

export function deleteUnit(id: number) {
	return apiDelete<MessageResponse>(`/units/${id}`);
}

function normalizePage(value: number | undefined) {
	if (!Number.isFinite(value)) return 1;
	return Math.max(1, Math.trunc(value ?? 1));
}

function normalizeLimit(value: number | undefined) {
	if (!Number.isFinite(value)) return 10;
	return Math.min(100, Math.max(1, Math.trunc(value ?? 10)));
}
