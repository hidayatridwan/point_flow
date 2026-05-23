import { clearAccessToken, setAccessToken } from './auth.svelte.js';
import { apiGet, apiPost } from './client.js';

export type LoginCredentials = {
	email: string;
	password: string;
};

export type AuthUser = {
	id: number;
	name: string;
	email: string;
	role: string;
	is_active: boolean;
	last_login_at: string | null;
	created_at: string;
	updated_at: string;
	deleted_at: string | null;
};

export type LoginSession = {
	access_token: string;
	token_type: string;
	expires_in: number;
	user: AuthUser;
};

export type MessageResponse = {
	message: string;
};

export async function login(credentials: LoginCredentials) {
	clearAccessToken();
	const session = await apiPost<LoginSession, LoginCredentials>('/auth/login', credentials, {
		retryOnUnauthorized: false
	});
	setAccessToken(session.access_token);
	return session;
}

export function getCurrentUser() {
	return apiGet<AuthUser>('/auth/me');
}

export async function logout() {
	try {
		return await apiPost<MessageResponse>('/auth/logout', undefined, {
			retryOnUnauthorized: false
		});
	} finally {
		clearAccessToken();
	}
}
