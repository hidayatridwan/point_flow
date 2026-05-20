type AuthState = {
	accessToken: string | null;
};

export const authState = $state<AuthState>({
	accessToken: null
});

export function getAccessToken() {
	return authState.accessToken;
}

export function setAccessToken(accessToken: string) {
	authState.accessToken = accessToken;
}

export function clearAccessToken() {
	authState.accessToken = null;
}
