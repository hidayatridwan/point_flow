import { apiDelete, apiGet, apiGetEnvelope, apiPost, apiPut } from './client.js';
import type { PaginationMeta } from './types.js';

export type User = {
	id: number;
	name: string;
	email: string;
	role_id: number;
	role_code: string;
	role_name: string;
	role: string;
	permissions: string[];
	is_active: boolean;
	last_login_at: string | null;
	created_at: string;
	updated_at: string;
	deleted_at: string | null;
};

export type UserListParams = {
	page?: number;
	limit?: number;
	search?: string;
};

export type UserPagination = {
	page: number;
	pageSize: number;
	totalRecords: number;
	totalPages: number;
};

export type CreateUserPayload = {
	name: string;
	email: string;
	password: string;
	role_id?: number;
	role?: string;
	is_active?: boolean;
};

export type UpdateUserPayload = {
	name?: string;
	email?: string;
	password?: string;
	role_id?: number;
	role?: string;
	is_active?: boolean;
};

export type MessageResponse = {
	message: string;
};

export type UserListResult = {
	users: User[];
	pagination: UserPagination;
	requestId: string;
};

export async function listUsers(params: UserListParams = {}): Promise<UserListResult> {
	const limit = normalizeLimit(params.limit);
	const page = normalizePage(params.page);
	const search = params.search?.trim();

	const envelope = await apiGetEnvelope<User[], PaginationMeta>('/users', {
		params: {
			limit: 100,
			offset: 0,
			search
		}
	});
	const allUsers = envelope.data ?? [];
	const users = allUsers.slice((page - 1) * limit, page * limit);
	const meta = envelope.meta ?? {
		page,
		limit,
		total_records: allUsers.length,
		total_pages: allUsers.length ? Math.max(1, Math.ceil(allUsers.length / limit)) : 0
	};

	return {
		users,
		pagination: {
			page: meta.page,
			pageSize: meta.limit,
			totalRecords: meta.total_records,
			totalPages: meta.total_pages
		},
		requestId: envelope.request_id
	};
}

export function getUser(id: number) {
	return apiGet<User>(`/users/${id}`);
}

export function createUser(payload: CreateUserPayload) {
	return apiPost<User, CreateUserPayload>('/users', payload);
}

export function updateUser(id: number, payload: UpdateUserPayload) {
	return apiPut<User, UpdateUserPayload>(`/users/${id}`, payload);
}

export function deleteUser(id: number) {
	return apiDelete<MessageResponse>(`/users/${id}`);
}

function normalizePage(value: number | undefined) {
	if (!Number.isFinite(value)) return 1;
	return Math.max(1, Math.trunc(value ?? 1));
}

function normalizeLimit(value: number | undefined) {
	if (!Number.isFinite(value)) return 10;
	return Math.min(100, Math.max(1, Math.trunc(value ?? 10)));
}
