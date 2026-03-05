import type { ApiError } from '$lib/types';

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 'http://localhost:8000';
const TOKEN_KEY = 'musicum_auth_token';

export function getToken(): string | null {
	if (typeof localStorage === 'undefined') return null;
	return localStorage.getItem(TOKEN_KEY);
}

export function setToken(token: string): void {
	if (typeof localStorage === 'undefined') return;
	localStorage.setItem(TOKEN_KEY, token);
}

export function clearToken(): void {
	if (typeof localStorage === 'undefined') return;
	localStorage.removeItem(TOKEN_KEY);
}

interface RequestOptions {
	requireAuth?: boolean;
}

async function handleResponse<T>(response: Response): Promise<T> {
	if (response.status === 204) {
		return undefined as T;
	}

	if (!response.ok) {
		let detail = 'An error occurred';
		try {
			const errorData = await response.json();
			detail = errorData.detail || detail;
		} catch {
			// Response body is not JSON
		}
		const error: ApiError = {
			detail,
			status: response.status
		};
		throw error;
	}

	return response.json();
}

function buildHeaders(requireAuth: boolean, isFormData: boolean = false): HeadersInit {
	const headers: HeadersInit = {};

	if (!isFormData) {
		headers['Content-Type'] = 'application/json';
	}

	if (requireAuth) {
		const token = getToken();
		if (token) {
			headers['Authorization'] = `Bearer ${token}`;
		}
	}

	return headers;
}

export async function get<T>(endpoint: string, options: RequestOptions = {}): Promise<T> {
	const { requireAuth = false } = options;

	const response = await fetch(`${API_BASE_URL}${endpoint}`, {
		method: 'GET',
		headers: buildHeaders(requireAuth)
	});

	return handleResponse<T>(response);
}

export async function post<T>(
	endpoint: string,
	data?: unknown,
	options: RequestOptions = {}
): Promise<T> {
	const { requireAuth = false } = options;

	const response = await fetch(`${API_BASE_URL}${endpoint}`, {
		method: 'POST',
		headers: buildHeaders(requireAuth),
		body: data ? JSON.stringify(data) : undefined
	});

	return handleResponse<T>(response);
}

export async function postFormData<T>(
	endpoint: string,
	formData: FormData,
	options: RequestOptions = {}
): Promise<T> {
	const { requireAuth = false } = options;

	const response = await fetch(`${API_BASE_URL}${endpoint}`, {
		method: 'POST',
		headers: buildHeaders(requireAuth, true),
		body: formData
	});

	return handleResponse<T>(response);
}

export async function patch<T>(
	endpoint: string,
	data?: unknown,
	options: RequestOptions = {}
): Promise<T> {
	const { requireAuth = false } = options;

	const response = await fetch(`${API_BASE_URL}${endpoint}`, {
		method: 'PATCH',
		headers: buildHeaders(requireAuth),
		body: data ? JSON.stringify(data) : undefined
	});

	return handleResponse<T>(response);
}

export async function put<T>(
	endpoint: string,
	data?: unknown,
	options: RequestOptions = {}
): Promise<T> {
	const { requireAuth = false } = options;

	const response = await fetch(`${API_BASE_URL}${endpoint}`, {
		method: 'PUT',
		headers: buildHeaders(requireAuth),
		body: data ? JSON.stringify(data) : undefined
	});

	return handleResponse<T>(response);
}

export async function del<T>(endpoint: string, options: RequestOptions = {}): Promise<T> {
	const { requireAuth = false } = options;

	const response = await fetch(`${API_BASE_URL}${endpoint}`, {
		method: 'DELETE',
		headers: buildHeaders(requireAuth)
	});

	return handleResponse<T>(response);
}
