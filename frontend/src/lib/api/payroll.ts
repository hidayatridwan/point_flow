import { apiGet, apiGetEnvelope, apiPost } from './client.js';
import type { PaginationMeta } from './types.js';

export type PayrollSlip = {
	id: number;
	slip_no: string;
	employee_id: number;
	employee_name: string;
	employee_position: string;
	period_start: string;
	period_end: string;
	base_salary: number;
	bonus: number;
	deduction: number;
	net_salary: number;
	status: string;
	cancelled_at: string | null;
	cancelled_by: number | null;
	cancel_reason: string | null;
	created_by: number | null;
	created_at: string;
	updated_at: string;
};

export type CreatePayrollSlipPayload = {
	employee_id: number;
	period_start: string;
	period_end: string;
	bonus?: number;
	deduction?: number;
};

export type PayrollSlipListParams = {
	page?: number;
	limit?: number;
	search?: string;
	from?: string;
	to?: string;
};

export type PayrollReportListParams = {
	page?: number;
	limit?: number;
	from?: string;
	to?: string;
};

export type PayrollPagination = {
	page: number;
	pageSize: number;
	totalRecords: number;
	totalPages: number;
};

export type PayrollSlipListResult = {
	payrollSlips: PayrollSlip[];
	pagination: PayrollPagination;
	requestId: string;
};

export type PayrollReport = {
	period_start: string;
	period_end: string;
	slip_count: number;
	total_base_salary: number;
	total_bonus: number;
	total_deduction: number;
	total_net_salary: number;
};

export type PayrollReportListResult = {
	reports: PayrollReport[];
	pagination: PayrollPagination;
	requestId: string;
};

export async function listPayrollSlips(
	params: PayrollSlipListParams = {}
): Promise<PayrollSlipListResult> {
	const limit = normalizeLimit(params.limit);
	const page = normalizePage(params.page);
	const search = params.search?.trim();
	const from = params.from?.trim();
	const to = params.to?.trim();

	const envelope = await apiGetEnvelope<PayrollSlip[], PaginationMeta>('/payroll-slips', {
		params: {
			limit,
			offset: (page - 1) * limit,
			search,
			from,
			to
		}
	});
	const payrollSlips = envelope.data ?? [];
	const meta = envelope.meta ?? getFallbackMeta(page, limit, payrollSlips.length);

	return {
		payrollSlips,
		pagination: {
			page: meta.page,
			pageSize: meta.limit,
			totalRecords: meta.total_records,
			totalPages: meta.total_pages
		},
		requestId: envelope.request_id
	};
}

export function getPayrollSlip(id: number) {
	return apiGet<PayrollSlip>(`/payroll-slips/${id}`);
}

export function createPayrollSlip(payload: CreatePayrollSlipPayload) {
	return apiPost<PayrollSlip, CreatePayrollSlipPayload>('/payroll-slips', payload);
}

export type CancelPayrollSlipPayload = {
	reason?: string | null;
};

export function cancelPayrollSlip(id: number, payload: CancelPayrollSlipPayload = {}) {
	return apiPost<PayrollSlip, CancelPayrollSlipPayload>(`/payroll-slips/${id}/cancel`, payload);
}

export async function listPayrollReport(
	params: PayrollReportListParams = {}
): Promise<PayrollReportListResult> {
	const limit = normalizeLimit(params.limit);
	const page = normalizePage(params.page);
	const from = params.from?.trim();
	const to = params.to?.trim();

	const envelope = await apiGetEnvelope<PayrollReport[], PaginationMeta>('/reports/payroll', {
		params: {
			limit,
			offset: (page - 1) * limit,
			from,
			to
		}
	});
	const reports = envelope.data ?? [];
	const meta = envelope.meta ?? getFallbackMeta(page, limit, reports.length);

	return {
		reports,
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
