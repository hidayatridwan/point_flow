import { apiDelete, apiGet, apiGetEnvelope, apiPost, apiPut } from './client.js';
import type { PaginationMeta } from './types.js';

export type FinancialTransactionType = 'income' | 'expense';
export const financialTransactionTypes: FinancialTransactionType[] = ['income', 'expense'];

export type FinancialTransactionStatus = 'completed' | 'cancelled';
export const financialTransactionStatuses: FinancialTransactionStatus[] = [
	'completed',
	'cancelled'
];

export type FinancialCategory = {
	id: number;
	name: string;
	category_type: FinancialTransactionType;
	description: string | null;
	/** `false` = hanya menggeser kas, tidak dihitung di Laba Rugi (mis. modal). */
	affects_profit: boolean;
	is_active: boolean;
	created_by: number | null;
	created_at: string;
	updated_at: string;
};

export type FinancialCategoryListParams = {
	page?: number;
	limit?: number;
	search?: string;
};

export type FinancialCategoryPagination = {
	page: number;
	pageSize: number;
	totalRecords: number;
	totalPages: number;
};

export type CreateFinancialCategoryPayload = {
	name: string;
	category_type: FinancialTransactionType;
	description?: string | null;
	affects_profit?: boolean;
	is_active?: boolean;
};

export type UpdateFinancialCategoryPayload = {
	name?: string;
	description?: string | null;
	affects_profit?: boolean;
	is_active?: boolean;
};

export type FinancialCategoryListResult = {
	categories: FinancialCategory[];
	pagination: FinancialCategoryPagination;
	requestId: string;
};

export type FinancialTransaction = {
	id: number;
	transaction_no: string;
	occurred_at: string;
	transaction_type: FinancialTransactionType;
	financial_category_id: number;
	financial_category_name: string;
	amount: number;
	is_cash: boolean;
	description: string | null;
	status: FinancialTransactionStatus;
	cancelled_at: string | null;
	cancelled_by: number | null;
	cancel_reason: string | null;
	recorded_by: number;
	created_at: string;
	updated_at: string;
};

export type FinancialTransactionListParams = {
	page?: number;
	limit?: number;
	search?: string;
	from?: string;
	to?: string;
	type?: FinancialTransactionType;
	financial_category_id?: number;
	status?: FinancialTransactionStatus[];
};

export type FinancialTransactionPagination = {
	page: number;
	pageSize: number;
	totalRecords: number;
	totalPages: number;
};

export type CreateFinancialTransactionPayload = {
	occurred_at?: string | null;
	transaction_type: FinancialTransactionType;
	financial_category_id: number;
	amount: number;
	is_cash?: boolean;
	description?: string | null;
};

export type UpdateFinancialTransactionPayload = {
	occurred_at?: string | null;
	financial_category_id?: number;
	amount?: number;
	is_cash?: boolean;
	description?: string | null;
};

export type CancelFinancialTransactionPayload = {
	reason?: string | null;
};

export type MessageResponse = {
	message: string;
};

export type FinancialTransactionListResult = {
	transactions: FinancialTransaction[];
	pagination: FinancialTransactionPagination;
	requestId: string;
};

export async function listFinancialCategories(
	params: FinancialCategoryListParams = {}
): Promise<FinancialCategoryListResult> {
	const limit = normalizeLimit(params.limit);
	const page = normalizePage(params.page);
	const search = params.search?.trim();

	const envelope = await apiGetEnvelope<FinancialCategory[], PaginationMeta>(
		'/financial-categories',
		{
			params: {
				limit,
				offset: (page - 1) * limit,
				search
			}
		}
	);
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

export function getFinancialCategory(id: number) {
	return apiGet<FinancialCategory>(`/financial-categories/${id}`);
}

export function createFinancialCategory(payload: CreateFinancialCategoryPayload) {
	return apiPost<FinancialCategory, CreateFinancialCategoryPayload>(
		'/financial-categories',
		payload
	);
}

export function updateFinancialCategory(id: number, payload: UpdateFinancialCategoryPayload) {
	return apiPut<FinancialCategory, UpdateFinancialCategoryPayload>(
		`/financial-categories/${id}`,
		payload
	);
}

export function deleteFinancialCategory(id: number) {
	return apiDelete<MessageResponse>(`/financial-categories/${id}`);
}

export async function listFinancialTransactions(
	params: FinancialTransactionListParams = {}
): Promise<FinancialTransactionListResult> {
	const limit = normalizeLimit(params.limit);
	const page = normalizePage(params.page);
	const search = params.search?.trim();
	const from = params.from?.trim();
	const to = params.to?.trim();
	const status = normalizeStatuses(params.status);

	const envelope = await apiGetEnvelope<FinancialTransaction[], PaginationMeta>(
		'/financial-transactions',
		{
			params: {
				limit,
				offset: (page - 1) * limit,
				search,
				from,
				to,
				type: params.type,
				financial_category_id: params.financial_category_id,
				status: status.length ? status.join(',') : undefined
			}
		}
	);
	const meta = envelope.meta ?? {
		page,
		limit,
		total_records: envelope.data?.length ?? 0,
		total_pages: envelope.data?.length ? 1 : 0
	};

	return {
		transactions: envelope.data ?? [],
		pagination: {
			page: meta.page,
			pageSize: meta.limit,
			totalRecords: meta.total_records,
			totalPages: meta.total_pages
		},
		requestId: envelope.request_id
	};
}

export function getFinancialTransaction(id: number) {
	return apiGet<FinancialTransaction>(`/financial-transactions/${id}`);
}

export function createFinancialTransaction(payload: CreateFinancialTransactionPayload) {
	return apiPost<FinancialTransaction, CreateFinancialTransactionPayload>(
		'/financial-transactions',
		payload
	);
}

export function updateFinancialTransaction(id: number, payload: UpdateFinancialTransactionPayload) {
	return apiPut<FinancialTransaction, UpdateFinancialTransactionPayload>(
		`/financial-transactions/${id}`,
		payload
	);
}

export function cancelFinancialTransaction(id: number, payload: CancelFinancialTransactionPayload) {
	return apiPost<FinancialTransaction, CancelFinancialTransactionPayload>(
		`/financial-transactions/${id}/cancel`,
		payload
	);
}

function normalizeStatuses(value: FinancialTransactionStatus[] | undefined) {
	if (!value) return [];
	return financialTransactionStatuses.filter((status) => value.includes(status));
}

function normalizePage(value: number | undefined) {
	if (!Number.isFinite(value)) return 1;
	return Math.max(1, Math.trunc(value ?? 1));
}

function normalizeLimit(value: number | undefined) {
	if (!Number.isFinite(value)) return 10;
	return Math.min(100, Math.max(1, Math.trunc(value ?? 10)));
}
