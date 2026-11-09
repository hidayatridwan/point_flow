import {
	changeMemberPassword,
	getMemberProfile,
	memberLogin,
	memberLogout,
	postPublicOrder,
	refreshMemberSession,
	type CheckoutInput,
	type CheckoutResult,
	type MemberProfile
} from './storefront.js';
import { ApiError } from './types.js';

type MemberAuthState = {
	accessToken: string | null;
	member: MemberProfile | null;
};

export const memberAuthState = $state<MemberAuthState>({
	accessToken: null,
	member: null
});

export function getMemberAccessToken() {
	return memberAuthState.accessToken;
}

function setSession(accessToken: string, member: MemberProfile) {
	memberAuthState.accessToken = accessToken;
	memberAuthState.member = member;
}

export function clearMemberSession() {
	memberAuthState.accessToken = null;
	memberAuthState.member = null;
}

export async function loginMember(phone: string, password: string): Promise<MemberProfile> {
	const session = await memberLogin(phone.trim(), password);
	setSession(session.access_token, session.member);
	return session.member;
}

export async function logoutMember(): Promise<void> {
	await memberLogout();
	clearMemberSession();
}

/** Coba pulihkan sesi dari cookie refresh (dipanggil sekali saat layout publik mount). */
export async function restoreMemberSession(): Promise<MemberProfile | null> {
	if (memberAuthState.accessToken) return memberAuthState.member;
	try {
		const refreshed = await refreshMemberSession();
		if (!refreshed) return null;
		const member = await getMemberProfile(refreshed);
		setSession(refreshed, member);
		return member;
	} catch {
		return null;
	}
}

async function withMemberRefresh<T>(call: (token: string | null) => Promise<T>): Promise<T> {
	try {
		return await call(memberAuthState.accessToken);
	} catch (error) {
		if (error instanceof ApiError && error.status === 401 && memberAuthState.accessToken) {
			const refreshed = await refreshMemberSession();
			if (refreshed) {
				memberAuthState.accessToken = refreshed;
				return call(refreshed);
			}
			clearMemberSession();
		}
		throw error;
	}
}

export function fetchMemberProfile(): Promise<MemberProfile> {
	return withMemberRefresh(async (token) => {
		const member = await getMemberProfile(token);
		memberAuthState.member = member;
		return member;
	});
}

export async function updateMemberPassword(
	currentPassword: string,
	newPassword: string
): Promise<void> {
	// Backend mencabut semua refresh token + cookie; sesi lokal ikut dibersihkan.
	await withMemberRefresh((token) => changeMemberPassword(token, currentPassword, newPassword));
	clearMemberSession();
}

export function submitMemberOrder(input: CheckoutInput): Promise<CheckoutResult> {
	return withMemberRefresh((token) => postPublicOrder(input, token));
}
