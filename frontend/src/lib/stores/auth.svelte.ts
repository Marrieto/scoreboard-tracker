// lib/stores/auth.ts — Authentication state store using Svelte 5 runes.

import { getAuthInfo, type AuthInfo } from '$lib/api';

let authState = $state<AuthInfo>({ authenticated: false });
let loading = $state(true);

export function getAuth() {
	return {
		get info() { return authState; },
		get loading() { return loading; },
	};
}

export async function checkAuth() {
	try {
		authState = await getAuthInfo();
	} catch {
		authState = { authenticated: false };
	} finally {
		loading = false;
	}
}
