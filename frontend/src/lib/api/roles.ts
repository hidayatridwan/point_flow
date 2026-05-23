import { apiDelete, apiGet, apiGetEnvelope, apiPost, apiPut } from './client.js';
import type { PaginationMeta } from './types.js';

export type Role = {
	id: number;
	code: string;
	name: string;
	description: string | null;
	is_system: boolean;
	permissions: string[];
	created_at: string;
	updated_at: string;
	deleted_at: string | null;
};

export type Permission = {
	id: number;
	code: string;
	name: string;
	description: string | null;
	created_at: string;
	updated_at: string;
};

export type RoleListParams = {
	page?: number;
	limit?: number;
	search?: string;
};

export type PermissionListParams = {
	limit?: number;
	search?: string;
};

export type RolePagination = {
	page: number;
	pageSize: number;
	totalRecords: number;
	totalPages: number;
};

export type CreateRolePayload = {
	code: string;
	name: string;
	description?: string | null;
	permission_codes?: string[];
};

export type UpdateRolePayload = {
	code?: string;
	name?: string;
	description?: string | null;
};

export type UpdateRolePermissionsPayload = {
	permission_codes: string[];
};

export type MessageResponse = {
	message: string;
};

export type RoleListResult = {
	roles: Role[];
	pagination: RolePagination;
	requestId: string;
};

export async function listRoles(params: RoleListParams = {}): Promise<RoleListResult> {
	const limit = normalizeLimit(params.limit);
	const page = normalizePage(params.page);
	const search = params.search?.trim();

	const envelope = await apiGetEnvelope<Role[], PaginationMeta>('/roles', {
		params: {
			limit: 100,
			offset: 0,
			search
		}
	});
	const allRoles = envelope.data ?? [];
	const roles = allRoles.slice((page - 1) * limit, page * limit);
	const meta = envelope.meta ?? {
		page,
		limit,
		total_records: allRoles.length,
		total_pages: allRoles.length ? Math.max(1, Math.ceil(allRoles.length / limit)) : 0
	};

	return {
		roles,
		pagination: {
			page: meta.page,
			pageSize: meta.limit,
			totalRecords: meta.total_records,
			totalPages: meta.total_pages
		},
		requestId: envelope.request_id
	};
}

export function getRole(id: number) {
	return apiGet<Role>(`/roles/${id}`);
}

export function createRole(payload: CreateRolePayload) {
	return apiPost<Role, CreateRolePayload>('/roles', payload);
}

export function updateRole(id: number, payload: UpdateRolePayload) {
	return apiPut<Role, UpdateRolePayload>(`/roles/${id}`, payload);
}

export function deleteRole(id: number) {
	return apiDelete<MessageResponse>(`/roles/${id}`);
}

export async function listPermissions(params: PermissionListParams = {}) {
	const envelope = await apiGetEnvelope<Permission[]>('/permissions', {
		params: {
			limit: normalizeLimit(params.limit ?? 100),
			search: params.search?.trim()
		}
	});

	return envelope.data ?? [];
}

export function replaceRolePermissions(id: number, payload: UpdateRolePermissionsPayload) {
	return apiPut<Role, UpdateRolePermissionsPayload>(`/roles/${id}/permissions`, payload);
}

function normalizePage(value: number | undefined) {
	if (!Number.isFinite(value)) return 1;
	return Math.max(1, Math.trunc(value ?? 1));
}

function normalizeLimit(value: number | undefined) {
	if (!Number.isFinite(value)) return 10;
	return Math.min(100, Math.max(1, Math.trunc(value ?? 10)));
}
