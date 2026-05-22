import { apiDelete, apiGet, apiGetEnvelope, apiPost, apiPut } from './client.js';
import type { PaginationMeta } from './types.js';

export type Employee = {
	id: number;
	name: string;
	position: string;
	phone: string | null;
	address: string | null;
	base_salary: number;
	is_active: boolean;
	created_at: string;
	updated_at: string;
	deleted_at: string | null;
};

export type EmployeeListParams = {
	page?: number;
	limit?: number;
	search?: string;
};

export type EmployeePagination = {
	page: number;
	pageSize: number;
	totalRecords: number;
	totalPages: number;
};

export type CreateEmployeePayload = {
	name: string;
	position: string;
	phone?: string | null;
	address?: string | null;
	base_salary: number;
	is_active?: boolean;
};

export type UpdateEmployeePayload = {
	name?: string;
	position?: string;
	phone?: string | null;
	address?: string | null;
	base_salary?: number;
	is_active?: boolean;
};

export type MessageResponse = {
	message: string;
};

export type EmployeeListResult = {
	employees: Employee[];
	pagination: EmployeePagination;
	requestId: string;
};

export async function listEmployees(params: EmployeeListParams = {}): Promise<EmployeeListResult> {
	const limit = normalizeLimit(params.limit);
	const page = normalizePage(params.page);
	const search = params.search?.trim();

	const envelope = await apiGetEnvelope<Employee[], PaginationMeta>('/employees', {
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
		employees: envelope.data ?? [],
		pagination: {
			page: meta.page,
			pageSize: meta.limit,
			totalRecords: meta.total_records,
			totalPages: meta.total_pages
		},
		requestId: envelope.request_id
	};
}

export function getEmployee(id: number) {
	return apiGet<Employee>(`/employees/${id}`);
}

export function createEmployee(payload: CreateEmployeePayload) {
	return apiPost<Employee, CreateEmployeePayload>('/employees', payload);
}

export function updateEmployee(id: number, payload: UpdateEmployeePayload) {
	return apiPut<Employee, UpdateEmployeePayload>(`/employees/${id}`, payload);
}

export function deleteEmployee(id: number) {
	return apiDelete<MessageResponse>(`/employees/${id}`);
}

function normalizePage(value: number | undefined) {
	if (!Number.isFinite(value)) return 1;
	return Math.max(1, Math.trunc(value ?? 1));
}

function normalizeLimit(value: number | undefined) {
	if (!Number.isFinite(value)) return 10;
	return Math.min(100, Math.max(1, Math.trunc(value ?? 10)));
}
