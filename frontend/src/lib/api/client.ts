import { env } from '$env/dynamic/public';
import { clearAccessToken, getAccessToken, setAccessToken } from './auth.svelte.js';
import { ApiError, type ApiEnvelope, type ApiErrorPayload } from './types.js';

const DEFAULT_API_BASE_URL = 'http://localhost:8080/api/v1';
const API_BASE_URL = (env.PUBLIC_API_BASE_URL || DEFAULT_API_BASE_URL).replace(/\/$/, '');

type ApiQueryParamPrimitive = string | number | boolean;
type ApiQueryParamValue = ApiQueryParamPrimitive | null | undefined | ApiQueryParamPrimitive[];
type ApiQueryParams = Record<string, ApiQueryParamValue>;

type ApiRequestOptions = Omit<RequestInit, 'body' | 'method'> & {
	body?: unknown;
	params?: ApiQueryParams;
	retryOnUnauthorized?: boolean;
	unwrapEnvelope?: boolean;
	/**
	 * Token eksplisit per request. `undefined` (default) memakai token staff
	 * global; string memakai token itu; `null` tidak mengirim header
	 * Authorization sama sekali (untuk endpoint publik/member).
	 */
	authToken?: string | null;
};

type RefreshTokenResponse = {
	access_token: string;
	token_type: string;
	expires_in: number;
	user: unknown;
};

let refreshPromise: Promise<string | null> | null = null;

export function apiGet<TResponse>(path: string, options: ApiRequestOptions = {}) {
	return apiRequest<TResponse>('GET', path, options);
}

export function apiGetEnvelope<TData, TMeta = null>(path: string, options: ApiRequestOptions = {}) {
	return apiRequest<ApiEnvelope<TData, TMeta>>('GET', path, { ...options, unwrapEnvelope: false });
}

export function apiPost<TResponse, TBody = unknown>(
	path: string,
	body?: TBody,
	options: ApiRequestOptions = {}
) {
	return apiRequest<TResponse>('POST', path, { ...options, body });
}

export function apiPut<TResponse, TBody = unknown>(
	path: string,
	body?: TBody,
	options: ApiRequestOptions = {}
) {
	return apiRequest<TResponse>('PUT', path, { ...options, body });
}

export function apiDelete<TResponse>(path: string, options: ApiRequestOptions = {}) {
	return apiRequest<TResponse>('DELETE', path, options);
}

export async function refreshAccessToken() {
	if (refreshPromise) return refreshPromise;

	refreshPromise = (async () => {
		try {
			const response = await fetch(buildUrl('/auth/refresh'), {
				method: 'POST',
				credentials: 'include',
				headers: {
					Accept: 'application/json'
				}
			});
			const session = await parseResponse<RefreshTokenResponse>(response);
			setAccessToken(session.access_token);
			return session.access_token;
		} catch {
			clearAccessToken();
			return null;
		} finally {
			refreshPromise = null;
		}
	})();

	return refreshPromise;
}

async function apiRequest<TResponse>(
	method: string,
	path: string,
	options: ApiRequestOptions,
	hasRetried = false
): Promise<TResponse> {
	const {
		body,
		headers,
		params,
		retryOnUnauthorized,
		unwrapEnvelope,
		authToken,
		...requestOptions
	} = options;

	const response = await fetch(buildUrl(path, params), {
		...requestOptions,
		method,
		credentials: 'include',
		headers: buildHeaders(body, headers, authToken),
		body: encodeBody(body)
	});

	if (response.status === 401 && retryOnUnauthorized !== false && !hasRetried) {
		const nextToken = await refreshAccessToken();
		if (nextToken) return apiRequest<TResponse>(method, path, options, true);
	}

	return parseResponse<TResponse>(response, unwrapEnvelope !== false);
}

export async function apiDownloadBlob(
	path: string,
	options: ApiRequestOptions = {}
): Promise<Blob> {
	const {
		params,
		headers,
		authToken,
		body,
		retryOnUnauthorized,
		unwrapEnvelope,
		...requestOptions
	} = options;
	void body;
	void retryOnUnauthorized;
	void unwrapEnvelope;
	const response = await fetch(buildUrl(path, params), {
		...requestOptions,
		method: 'GET',
		credentials: 'include',
		headers: buildHeaders(undefined, headers, authToken)
	});

	if (!response.ok) {
		throw await toApiError(response);
	}

	return response.blob();
}

async function toApiError(response: Response): Promise<ApiError> {
	const payload = await response
		.json()
		.then((value) => value as ApiEnvelope<unknown>)
		.catch(() => null);
	const body = payload?.error;
	return new ApiError(
		body?.message ?? 'Request failed. Please try again.',
		response.status,
		payload
	);
}

function buildUrl(path: string, params?: ApiQueryParams) {
	const url = /^https?:\/\//.test(path)
		? path
		: `${API_BASE_URL}${path.startsWith('/') ? path : `/${path}`}`;
	const query = buildQuery(params);

	if (!query) return url;

	const separator = url.includes('?') ? (url.endsWith('?') || url.endsWith('&') ? '' : '&') : '?';
	return `${url}${separator}${query}`;
}

function buildQuery(params: ApiQueryParams = {}) {
	const query = new URLSearchParams();

	for (const [key, value] of Object.entries(params)) {
		appendQueryParam(query, key, value);
	}

	return query.toString();
}

function appendQueryParam(query: URLSearchParams, key: string, value: ApiQueryParamValue) {
	if (Array.isArray(value)) {
		for (const item of value) {
			appendQueryParamValue(query, key, item);
		}
		return;
	}

	appendQueryParamValue(query, key, value);
}

function appendQueryParamValue(
	query: URLSearchParams,
	key: string,
	value: ApiQueryParamPrimitive | null | undefined
) {
	if (value === undefined || value === null || value === '') return;
	query.append(key, String(value));
}

function buildHeaders(body: unknown, headers: HeadersInit = {}, authToken?: string | null) {
	const nextHeaders = new Headers(headers);
	nextHeaders.set('Accept', 'application/json');

	if (isJsonBody(body) && !nextHeaders.has('Content-Type')) {
		nextHeaders.set('Content-Type', 'application/json');
	}

	// authToken === null: tanpa header Authorization (endpoint publik/member).
	// authToken === undefined: fallback ke token staff global.
	const accessToken = authToken === undefined ? getAccessToken() : authToken;
	if (accessToken) nextHeaders.set('Authorization', `Bearer ${accessToken}`);

	return nextHeaders;
}

function encodeBody(body: unknown) {
	if (body === undefined || body === null) return undefined;
	if (isJsonBody(body)) return JSON.stringify(body);
	return body as BodyInit;
}

function isJsonBody(body: unknown) {
	return (
		body !== undefined &&
		body !== null &&
		typeof body !== 'string' &&
		!(body instanceof FormData) &&
		!(body instanceof URLSearchParams) &&
		!(body instanceof Blob) &&
		!(body instanceof ArrayBuffer)
	);
}

async function parseResponse<TResponse>(
	response: Response,
	unwrapEnvelope = true
): Promise<TResponse> {
	const text = await response.text();
	const payload = parsePayload(text);

	if (!response.ok) {
		throw new ApiError(getErrorMessage(payload, response.statusText), response.status, payload);
	}

	if (unwrapEnvelope && isApiEnvelope(payload)) {
		return payload.data as TResponse;
	}

	return payload as TResponse;
}

function parsePayload(text: string) {
	if (!text) return null;

	try {
		return JSON.parse(text) as unknown;
	} catch {
		return text;
	}
}

function getErrorMessage(payload: unknown, fallback: string) {
	if (typeof payload === 'string') return payload || fallback;
	if (payload && typeof payload === 'object') {
		const body = payload as ApiErrorPayload;
		if (body.error && typeof body.error === 'object' && 'message' in body.error) {
			const message = body.error.message;
			return typeof message === 'string' && message ? message : fallback;
		}
		if (typeof body.error === 'string' && body.error) return body.error;
		if (typeof body.message === 'string' && body.message) return body.message;
	}
	return fallback;
}

function isApiEnvelope(payload: unknown): payload is ApiEnvelope<unknown, unknown> {
	if (!payload || typeof payload !== 'object') return false;
	const body = payload as Record<string, unknown>;
	return 'data' in body && 'meta' in body && 'error' in body && 'request_id' in body;
}
