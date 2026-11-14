import { apiDelete, apiGet, apiGetEnvelope, apiPost, apiPut } from './client.js';
import type { PaginationMeta } from './types.js';

const ACTIVE_SESSION_STORAGE_KEY = 'point_flow.active_cash_register_session_id';

export const cashRegisterStatuses = ['active', 'inactive'] as const;
export const cashRegisterSessionStatuses = ['open', 'closed'] as const;
export const cashDrawerMovementStatuses = ['completed', 'cancelled'] as const;
export const cashDrawerMovementTypes = ['cash_in', 'cash_out', 'cash_drop', 'petty_cash'] as const;
export const cashDrawerMovementDirections = ['in', 'out'] as const;

export type CashRegisterStatus = (typeof cashRegisterStatuses)[number];
export type CashRegisterSessionStatus = (typeof cashRegisterSessionStatuses)[number];
export type CashDrawerMovementStatus = (typeof cashDrawerMovementStatuses)[number];
export type CashDrawerMovementType = (typeof cashDrawerMovementTypes)[number];
export type CashDrawerMovementDirection = (typeof cashDrawerMovementDirections)[number];

export type CashRegister = {
	id: number;
	code: string;
	name: string;
	description: string | null;
	status: CashRegisterStatus;
	created_at: string;
	updated_at: string;
	deleted_at: string | null;
};

export type CashRegisterSession = {
	id: number;
	session_no: string;
	cash_register_id: number;
	cash_register_code: string;
	cash_register_name: string;
	opened_at: string;
	opened_by: number;
	opened_by_name: string;
	opening_cash: number;
	opening_notes: string | null;
	closed_at: string | null;
	closed_by: number | null;
	closed_by_name: string | null;
	closing_cash_counted: number | null;
	expected_cash: number | null;
	/** Estimasi live uang seharusnya; hanya terisi selama sesi masih `open`. */
	expected_cash_preview: number | null;
	cash_difference: number | null;
	closing_notes: string | null;
	status: CashRegisterSessionStatus;
	created_at: string;
	updated_at: string;
};

export type CashDrawerMovement = {
	id: number;
	movement_no: string;
	cash_register_session_id: number;
	session_no: string;
	cash_register_id: number;
	cash_register_code: string;
	cash_register_name: string;
	occurred_at: string;
	created_by: number;
	created_by_name: string;
	movement_type: CashDrawerMovementType;
	direction: CashDrawerMovementDirection;
	amount: number;
	reason: string | null;
	notes: string | null;
	status: CashDrawerMovementStatus;
	cancelled_at: string | null;
	cancelled_by: number | null;
	cancelled_by_name: string | null;
	cancel_reason: string | null;
	created_at: string;
	updated_at: string;
};

export type CashRegisterListParams = {
	page?: number;
	limit?: number;
	search?: string;
	status?: CashRegisterStatus[];
};

export type CashRegisterSessionListParams = {
	page?: number;
	limit?: number;
	search?: string;
	from?: string;
	to?: string;
	status?: CashRegisterSessionStatus[];
};

export type CashDrawerMovementListParams = {
	page?: number;
	limit?: number;
	search?: string;
	from?: string;
	to?: string;
	status?: CashDrawerMovementStatus[];
	type?: CashDrawerMovementType[];
	cash_register_session_id?: number | null;
};

export type CashRegisterPagination = {
	page: number;
	pageSize: number;
	totalRecords: number;
	totalPages: number;
};

export type CashRegisterListResult = {
	cashRegisters: CashRegister[];
	pagination: CashRegisterPagination;
	requestId: string;
};

export type CashRegisterSessionListResult = {
	sessions: CashRegisterSession[];
	pagination: CashRegisterPagination;
	requestId: string;
};

export type CashDrawerMovementListResult = {
	movements: CashDrawerMovement[];
	pagination: CashRegisterPagination;
	requestId: string;
};

export type CreateCashRegisterPayload = {
	code: string;
	name: string;
	description?: string | null;
	status?: CashRegisterStatus;
};

export type UpdateCashRegisterPayload = {
	code?: string;
	name?: string;
	description?: string | null;
	status?: CashRegisterStatus;
};

export type OpenCashRegisterSessionPayload = {
	cash_register_id: number;
	opening_cash: number;
	opening_notes?: string | null;
};

export type CloseCashRegisterSessionPayload = {
	closing_cash_counted: number;
	closing_notes?: string | null;
};

export type CreateCashDrawerMovementPayload = {
	cash_register_session_id: number;
	occurred_at?: string | null;
	movement_type: CashDrawerMovementType;
	amount: number;
	reason?: string | null;
	notes?: string | null;
};

export type CancelCashDrawerMovementPayload = {
	reason?: string | null;
};

export type MessageResponse = {
	message: string;
};

export async function listCashRegisters(
	params: CashRegisterListParams = {}
): Promise<CashRegisterListResult> {
	const limit = normalizeLimit(params.limit);
	const page = normalizePage(params.page);
	const search = params.search?.trim();
	const status = normalizeCashRegisterStatuses(params.status);

	const envelope = await apiGetEnvelope<CashRegister[], PaginationMeta>('/cash-registers', {
		params: {
			limit: 100,
			offset: 0,
			search,
			status: status.length ? status.join(',') : undefined
		}
	});
	const allCashRegisters = envelope.data ?? [];
	const cashRegisters = allCashRegisters.slice((page - 1) * limit, page * limit);
	const meta = getFallbackMeta(page, limit, allCashRegisters.length);

	return {
		cashRegisters,
		pagination: {
			page: meta.page,
			pageSize: meta.limit,
			totalRecords: meta.total_records,
			totalPages: meta.total_pages
		},
		requestId: envelope.request_id
	};
}

export function createCashRegister(payload: CreateCashRegisterPayload) {
	return apiPost<CashRegister, CreateCashRegisterPayload>('/cash-registers', payload);
}

export function updateCashRegister(id: number, payload: UpdateCashRegisterPayload) {
	return apiPut<CashRegister, UpdateCashRegisterPayload>(`/cash-registers/${id}`, payload);
}

export function deleteCashRegister(id: number) {
	return apiDelete<MessageResponse>(`/cash-registers/${id}`);
}

export async function listCashRegisterSessions(
	params: CashRegisterSessionListParams = {}
): Promise<CashRegisterSessionListResult> {
	const limit = normalizeLimit(params.limit);
	const page = normalizePage(params.page);
	const search = params.search?.trim();
	const from = params.from?.trim();
	const to = params.to?.trim();
	const status = normalizeCashRegisterSessionStatuses(params.status);

	const envelope = await apiGetEnvelope<CashRegisterSession[], PaginationMeta>(
		'/cash-register-sessions',
		{
			params: {
				limit: 100,
				offset: 0,
				search,
				from,
				to,
				status: status.length ? status.join(',') : undefined
			}
		}
	);
	const allSessions = envelope.data ?? [];
	const sessions = allSessions.slice((page - 1) * limit, page * limit);
	const meta = getFallbackMeta(page, limit, allSessions.length);

	return {
		sessions,
		pagination: {
			page: meta.page,
			pageSize: meta.limit,
			totalRecords: meta.total_records,
			totalPages: meta.total_pages
		},
		requestId: envelope.request_id
	};
}

export function openCashRegisterSession(payload: OpenCashRegisterSessionPayload) {
	return apiPost<CashRegisterSession, OpenCashRegisterSessionPayload>(
		'/cash-register-sessions/open',
		payload
	);
}

export function closeCashRegisterSession(id: number, payload: CloseCashRegisterSessionPayload) {
	return apiPost<CashRegisterSession, CloseCashRegisterSessionPayload>(
		`/cash-register-sessions/${id}/close`,
		payload
	);
}

export async function listCashDrawerMovements(
	params: CashDrawerMovementListParams = {}
): Promise<CashDrawerMovementListResult> {
	const limit = normalizeLimit(params.limit);
	const page = normalizePage(params.page);
	const search = params.search?.trim();
	const from = params.from?.trim();
	const to = params.to?.trim();
	const status = normalizeCashDrawerMovementStatuses(params.status);
	const types = normalizeCashDrawerMovementTypes(params.type);

	const envelope = await apiGetEnvelope<CashDrawerMovement[], PaginationMeta>(
		'/cash-drawer-movements',
		{
			params: {
				limit: 100,
				offset: 0,
				search,
				from,
				to,
				status: status.length ? status.join(',') : undefined,
				type: types.length ? types.join(',') : undefined,
				cash_register_session_id: params.cash_register_session_id ?? undefined
			}
		}
	);
	const allMovements = envelope.data ?? [];
	const movements = allMovements.slice((page - 1) * limit, page * limit);
	const meta = getFallbackMeta(page, limit, allMovements.length);

	return {
		movements,
		pagination: {
			page: meta.page,
			pageSize: meta.limit,
			totalRecords: meta.total_records,
			totalPages: meta.total_pages
		},
		requestId: envelope.request_id
	};
}

export function getCashDrawerMovement(id: number) {
	return apiGet<CashDrawerMovement>(`/cash-drawer-movements/${id}`);
}

export function createCashDrawerMovement(payload: CreateCashDrawerMovementPayload) {
	return apiPost<CashDrawerMovement, CreateCashDrawerMovementPayload>(
		'/cash-drawer-movements',
		payload
	);
}

export function cancelCashDrawerMovement(id: number, payload: CancelCashDrawerMovementPayload) {
	return apiPost<CashDrawerMovement, CancelCashDrawerMovementPayload>(
		`/cash-drawer-movements/${id}/cancel`,
		payload
	);
}

export function getStoredActiveCashRegisterSessionId() {
	if (typeof window === 'undefined') return null;
	const value = window.localStorage.getItem(ACTIVE_SESSION_STORAGE_KEY);
	const id = Number(value);
	return Number.isFinite(id) && id > 0 ? Math.trunc(id) : null;
}

export function setStoredActiveCashRegisterSessionId(id: number) {
	if (typeof window === 'undefined') return;
	window.localStorage.setItem(ACTIVE_SESSION_STORAGE_KEY, String(id));
}

export function clearStoredActiveCashRegisterSessionId() {
	if (typeof window === 'undefined') return;
	window.localStorage.removeItem(ACTIVE_SESSION_STORAGE_KEY);
}

function getFallbackMeta(page: number, limit: number, totalRecords: number): PaginationMeta {
	return {
		page,
		limit,
		total_records: totalRecords,
		total_pages: totalRecords ? Math.max(1, Math.ceil(totalRecords / limit)) : 0
	};
}

function normalizeCashRegisterStatuses(value: CashRegisterStatus[] | undefined) {
	if (!value?.length) return [];
	return value.filter(
		(status, index) => cashRegisterStatuses.includes(status) && value.indexOf(status) === index
	);
}

function normalizeCashRegisterSessionStatuses(value: CashRegisterSessionStatus[] | undefined) {
	if (!value?.length) return [];
	return value.filter(
		(status, index) =>
			cashRegisterSessionStatuses.includes(status) && value.indexOf(status) === index
	);
}

function normalizeCashDrawerMovementStatuses(value: CashDrawerMovementStatus[] | undefined) {
	if (!value?.length) return [];
	return value.filter(
		(status, index) =>
			cashDrawerMovementStatuses.includes(status) && value.indexOf(status) === index
	);
}

function normalizeCashDrawerMovementTypes(value: CashDrawerMovementType[] | undefined) {
	if (!value?.length) return [];
	return value.filter(
		(type, index) => cashDrawerMovementTypes.includes(type) && value.indexOf(type) === index
	);
}

function normalizePage(value: number | undefined) {
	if (!Number.isFinite(value)) return 1;
	return Math.max(1, Math.trunc(value ?? 1));
}

function normalizeLimit(value: number | undefined) {
	if (!Number.isFinite(value)) return 10;
	return Math.min(100, Math.max(1, Math.trunc(value ?? 10)));
}
