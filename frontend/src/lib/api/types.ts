export type ApiErrorPayload = {
	error?: string | ApiErrorDetail | null;
	message?: string;
	[key: string]: unknown;
};

export type ApiErrorDetail = {
	code?: string;
	message?: string;
	[key: string]: unknown;
};

export type ApiEnvelope<TData, TMeta = null> = {
	data: TData | null;
	meta: TMeta | null;
	error: ApiErrorDetail | null;
	request_id: string;
};

export type PaginationMeta = {
	page: number;
	limit: number;
	total_records: number;
	total_pages: number;
};

export class ApiError extends Error {
	status: number;
	payload: unknown;

	constructor(message: string, status: number, payload: unknown = null) {
		super(message);
		this.name = 'ApiError';
		this.status = status;
		this.payload = payload;
	}
}
