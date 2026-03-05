export enum UserRole {
	USER = 'user',
	ADMIN = 'admin',
	MODERATOR = 'moderator'
}

export interface UserResponse {
	id: number;
	username: string;
	email: string;
	role: UserRole;
	is_active: number;
	created_at: string;
	updated_at: string;
}

export interface Token {
	access_token: string;
	token_type: string;
}

export interface LoginRequest {
	email: string;
	password: string;
}

export interface UserCreate {
	username: string;
	email: string;
	password: string;
}
