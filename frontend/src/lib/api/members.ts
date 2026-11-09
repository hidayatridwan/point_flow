import { apiDelete, apiGet, apiGetEnvelope, apiPost, apiPut } from './client.js';
import type { PaginationMeta } from './types.js';

export type Member = {
	id: number;
	member_no: string;
	full_name: string;
	phone: string;
	email: string | null;
	address: string | null;
	joined_at: string;
	is_active: boolean;
	notes: string | null;
	created_at: string;
	updated_at: string;
};

export type MemberListParams = {
	search?: string;
	is_active?: boolean;
	limit?: number;
	offset?: number;
};

export type CreateMemberPayload = {
	full_name: string;
	phone: string;
	email?: string | null;
	password: string;
	address?: string | null;
	is_active?: boolean;
	notes?: string | null;
};

export type UpdateMemberPayload = {
	full_name?: string;
	phone?: string;
	email?: string | null;
	address?: string | null;
	is_active?: boolean;
	notes?: string | null;
};

export type ResetMemberPasswordPayload = {
	new_password: string;
};

export async function listMembers(params: MemberListParams = {}): Promise<Member[]> {
	const limit = normalizeLimit(params.limit);
	const offset = Math.max(0, params.offset ?? 0);
	const search = params.search?.trim();

	const envelope = await apiGetEnvelope<Member[], PaginationMeta>('/members', {
		params: {
			limit,
			offset,
			search,
			is_active: params.is_active
		}
	});
	return envelope.data ?? [];
}

export function getMember(id: number) {
	return apiGet<Member>(`/members/${id}`);
}

export function createMember(payload: CreateMemberPayload) {
	return apiPost<Member, CreateMemberPayload>('/members', payload);
}

export function updateMember(id: number, payload: UpdateMemberPayload) {
	return apiPut<Member, UpdateMemberPayload>(`/members/${id}`, payload);
}

export function deleteMember(id: number) {
	return apiDelete<Member>(`/members/${id}`);
}

export function resetMemberPassword(id: number, payload: ResetMemberPasswordPayload) {
	return apiPost<Member, ResetMemberPasswordPayload>(`/members/${id}/reset-password`, payload);
}

function normalizeLimit(value: number | undefined) {
	if (!Number.isFinite(value)) return 100;
	return Math.min(200, Math.max(1, Math.trunc(value ?? 100)));
}
