import type { Token, UserResponse, UserCreate, LoginRequest } from '$lib/types';
import { get, post, del, setToken } from './client';

export async function register(data: UserCreate): Promise<UserResponse> {
	return post<UserResponse>('/auth/register', data);
}

export async function login(data: LoginRequest): Promise<Token> {
	const token = await post<Token>('/auth/login', data);
	setToken(token.access_token);
	return token;
}

export async function getCurrentUser(): Promise<UserResponse> {
	return get<UserResponse>('/auth/me', { requireAuth: true });
}

export async function deleteUser(userId: number): Promise<void> {
	return del<void>(`/auth/users/${userId}`, { requireAuth: true });
}
