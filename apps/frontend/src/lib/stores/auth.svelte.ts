import type { UserResponse } from '$lib/types';
import { getToken, clearToken } from '$lib/api/client';
import { getCurrentUser } from '$lib/api/auth';

let user = $state<UserResponse | null>(null);
let isLoading = $state(false);

export const authStore = {
	get user() {
		return user;
	},
	get isLoading() {
		return isLoading;
	},
	get isAuthenticated() {
		return user !== null;
	},

	setUser(newUser: UserResponse | null) {
		user = newUser;
	},

	clearUser() {
		user = null;
		clearToken();
	},

	async initialize() {
		const token = getToken();
		if (!token) {
			user = null;
			return;
		}

		isLoading = true;
		try {
			user = await getCurrentUser();
		} catch {
			user = null;
			clearToken();
		} finally {
			isLoading = false;
		}
	}
};
