import { apiGet, apiGetEnvelope } from './client.js';
import type { PaginationMeta } from './types.js';

export const auditLogOutcomes = ['success', 'failure'] as const;
export const auditLogActions = [
	'auth.login',
	'auth.logout',
	'auth.bootstrap_admin',
	'auth.change_password',
	'cash_register_session.open',
	'cash_register_session.close',
	'cash_drawer_movement.create',
	'cash_drawer_movement.cancel',
	'sale.create',
	'sale.cancel',
	'sale_return.create',
	'sale_return.cancel'
] as const;
export const auditLogResourceTypes = [
	'user',
	'cash_register_session',
	'cash_drawer_movement',
	'sale',
	'sale_return'
] as const;

export type AuditLogOutcome = (typeof auditLogOutcomes)[number];
export type AuditLogAction = (typeof auditLogActions)[number];
export type AuditLogResourceType = (typeof auditLogResourceTypes)[number];
export type AuditJsonValue =
	| null
	| string
	| number
	| boolean
	| AuditJsonValue[]
	| { [key: string]: AuditJsonValue };

export type AuditLog = {
	id: number;
	request_id: string | null;
	actor_id: number | null;
	actor_name: string | null;
	actor_email: string | null;
	actor_role: string | null;
	action: string;
	resource_type: string;
	resource_id: number | null;
	resource_label: string | null;
	outcome: AuditLogOutcome;
	before_data: AuditJsonValue | null;
	after_data: AuditJsonValue | null;
	metadata: AuditJsonValue | null;
	ip_address: string | null;
	user_agent: string | null;
	occurred_at: string;
};

export type AuditLogListParams = {
	page?: number;
	limit?: number;
	search?: string;
	from?: string;
	to?: string;
	action?: AuditLogAction[];
	resource_type?: AuditLogResourceType[];
	outcome?: AuditLogOutcome[];
	actor_id?: number | null;
	request_id?: string;
};

export type AuditLogPagination = {
	page: number;
	pageSize: number;
	totalRecords: number;
	totalPages: number;
};

export type AuditLogListResult = {
	auditLogs: AuditLog[];
	pagination: AuditLogPagination;
	requestId: string;
};

export async function listAuditLogs(params: AuditLogListParams = {}): Promise<AuditLogListResult> {
	const limit = normalizeLimit(params.limit);
	const page = normalizePage(params.page);
	const search = params.search?.trim();
	const from = params.from?.trim();
	const to = params.to?.trim();
	const action = normalizeAuditLogActions(params.action);
	const resourceType = normalizeAuditLogResourceTypes(params.resource_type);
	const outcome = normalizeAuditLogOutcomes(params.outcome);
	const actorId =
		Number.isFinite(params.actor_id) && (params.actor_id ?? 0) > 0
			? Math.trunc(params.actor_id as number)
			: undefined;
	const requestId = params.request_id?.trim();

	const envelope = await apiGetEnvelope<AuditLog[], PaginationMeta>('/audit-logs', {
		params: {
			limit: 100,
			offset: 0,
			search,
			from,
			to,
			action: action.length ? action.join(',') : undefined,
			resource_type: resourceType.length ? resourceType.join(',') : undefined,
			outcome: outcome.length ? outcome.join(',') : undefined,
			actor_id: actorId,
			request_id: requestId
		}
	});
	const allAuditLogs = envelope.data ?? [];
	const auditLogs = allAuditLogs.slice((page - 1) * limit, page * limit);
	const meta = getFallbackMeta(page, limit, allAuditLogs.length);

	return {
		auditLogs,
		pagination: {
			page: meta.page,
			pageSize: meta.limit,
			totalRecords: meta.total_records,
			totalPages: meta.total_pages
		},
		requestId: envelope.request_id
	};
}

export function getAuditLog(id: number) {
	return apiGet<AuditLog>(`/audit-logs/${id}`);
}

function getFallbackMeta(page: number, limit: number, totalRecords: number): PaginationMeta {
	return {
		page,
		limit,
		total_records: totalRecords,
		total_pages: totalRecords ? Math.max(1, Math.ceil(totalRecords / limit)) : 0
	};
}

function normalizeAuditLogActions(value: AuditLogAction[] | undefined) {
	if (!value?.length) return [];
	return value.filter(
		(action, index) => auditLogActions.includes(action) && value.indexOf(action) === index
	);
}

function normalizeAuditLogResourceTypes(value: AuditLogResourceType[] | undefined) {
	if (!value?.length) return [];
	return value.filter(
		(resourceType, index) =>
			auditLogResourceTypes.includes(resourceType) && value.indexOf(resourceType) === index
	);
}

function normalizeAuditLogOutcomes(value: AuditLogOutcome[] | undefined) {
	if (!value?.length) return [];
	return value.filter(
		(outcome, index) => auditLogOutcomes.includes(outcome) && value.indexOf(outcome) === index
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
