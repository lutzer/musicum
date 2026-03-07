import type { UserListResponse, PaginationParams } from '$lib/types';
import { get } from './client';

function buildQueryString<T extends object>(params: T): string {
	const searchParams = new URLSearchParams();
	for (const [key, value] of Object.entries(params)) {
		if (value !== undefined && value !== null) {
			searchParams.append(key, String(value));
		}
	}
	const query = searchParams.toString();
	return query ? `?${query}` : '';
}

export async function listUsers(params?: PaginationParams): Promise<UserListResponse> {
	const query = params ? buildQueryString(params) : '';
	return get<UserListResponse>(`/users${query}`, { requireAuth: true });
}
