import { apiDownloadBlob, apiGet, apiGetEnvelope, apiPost, apiPut } from './client.js';
import type { PaginationMeta } from './types.js';

export const profitSharingPeriodStatuses = [
	'draft',
	'calculated',
	'finalized',
	'cancelled'
] as const;

export type ProfitSharingPeriodStatus = (typeof profitSharingPeriodStatuses)[number];

export type ProfitSharingPeriod = {
	id: number;
	period_name: string;
	start_date: string;
	end_date: string;
	status: ProfitSharingPeriodStatus;
	total_pool: number | null;
	auto_net_profit: number | null;
	member_pct: number;
	reserve_pct: number;
	management_pct: number;
	supervisor_pct: number;
	social_pct: number;
	education_pct: number;
	notes: string | null;
	calculated_at: string | null;
	finalized_at: string | null;
	finalized_by: number | null;
	cancelled_at: string | null;
	cancelled_by: number | null;
	cancel_reason: string | null;
	created_at: string;
	updated_at: string;
};

export type BucketAllocation = {
	key: string;
	percentage: number;
	amount: number;
};

export type ProfitSharingPeriodDetail = ProfitSharingPeriod & {
	pool_used: number;
	member_pool: number;
	member_count: number;
	total_member_transaction: number;
	allocations: BucketAllocation[];
};

export type ProfitSharingDistribution = {
	id: number;
	profit_sharing_period_id: number;
	member_id: number;
	member_no: string;
	full_name: string;
	phone: string;
	transaction_amount: number;
	share_pct: number;
	share_amount: number;
	calculated_at: string;
};

export type PeriodListParams = {
	page?: number;
	limit?: number;
	search?: string;
	status?: ProfitSharingPeriodStatus[];
};

export type DistributionListParams = {
	page?: number;
	limit?: number;
	search?: string;
};

export type CreatePeriodPayload = {
	period_name: string;
	start_date: string;
	end_date: string;
	total_pool?: number | null;
	member_pct: number;
	reserve_pct: number;
	management_pct: number;
	supervisor_pct: number;
	social_pct: number;
	education_pct: number;
	notes?: string | null;
};

export type UpdatePeriodPayload = {
	period_name?: string;
	start_date?: string;
	end_date?: string;
	total_pool?: number | null;
	member_pct?: number;
	reserve_pct?: number;
	management_pct?: number;
	supervisor_pct?: number;
	social_pct?: number;
	education_pct?: number;
	notes?: string | null;
};

export type CancelPeriodPayload = {
	reason?: string | null;
};

function toOffset(page: number | undefined, limit: number | undefined) {
	const safePage = page && page > 0 ? Math.trunc(page) : 1;
	const safeLimit = limit && limit > 0 ? Math.min(100, Math.trunc(limit)) : 20;
	return { limit: safeLimit, offset: (safePage - 1) * safeLimit };
}

export async function listPeriods(params: PeriodListParams = {}) {
	const { limit, offset } = toOffset(params.page, params.limit);
	const envelope = await apiGetEnvelope<ProfitSharingPeriod[], PaginationMeta>(
		'/profit-sharing/periods',
		{
			params: {
				limit,
				offset,
				search: params.search?.trim() || undefined,
				status: params.status
			}
		}
	);
	const meta = envelope.meta ?? {
		page: 1,
		limit,
		total_records: envelope.data?.length ?? 0,
		total_pages: envelope.data?.length ? 1 : 0
	};
	return {
		periods: envelope.data ?? [],
		pagination: {
			page: meta.page,
			pageSize: meta.limit,
			totalRecords: meta.total_records,
			totalPages: meta.total_pages
		}
	};
}

export function getPeriodDetail(id: number) {
	return apiGet<ProfitSharingPeriodDetail>(`/profit-sharing/periods/${id}`);
}

export function createPeriod(payload: CreatePeriodPayload) {
	return apiPost<ProfitSharingPeriodDetail, CreatePeriodPayload>(
		'/profit-sharing/periods',
		payload
	);
}

export function updatePeriod(id: number, payload: UpdatePeriodPayload) {
	return apiPut<ProfitSharingPeriodDetail, UpdatePeriodPayload>(
		`/profit-sharing/periods/${id}`,
		payload
	);
}

export function calculatePeriod(id: number) {
	return apiPost<ProfitSharingPeriodDetail>(`/profit-sharing/periods/${id}/calculate`);
}

export function finalizePeriod(id: number) {
	return apiPost<ProfitSharingPeriodDetail>(`/profit-sharing/periods/${id}/finalize`);
}

export function cancelPeriod(id: number, payload: CancelPeriodPayload) {
	return apiPost<ProfitSharingPeriodDetail, CancelPeriodPayload>(
		`/profit-sharing/periods/${id}/cancel`,
		payload
	);
}

export async function listDistributions(id: number, params: DistributionListParams = {}) {
	const { limit, offset } = toOffset(params.page, params.limit);
	const envelope = await apiGetEnvelope<ProfitSharingDistribution[], PaginationMeta>(
		`/profit-sharing/periods/${id}/distributions`,
		{
			params: {
				limit,
				offset,
				search: params.search?.trim() || undefined
			}
		}
	);
	const meta = envelope.meta ?? {
		page: 1,
		limit,
		total_records: envelope.data?.length ?? 0,
		total_pages: envelope.data?.length ? 1 : 0
	};
	return {
		distributions: envelope.data ?? [],
		pagination: {
			page: meta.page,
			pageSize: meta.limit,
			totalRecords: meta.total_records,
			totalPages: meta.total_pages
		}
	};
}

export function exportDistributionsCsv(id: number): Promise<Blob> {
	return apiDownloadBlob(`/profit-sharing/periods/${id}/distributions/export`);
}
