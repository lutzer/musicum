export interface ApiError {
	detail: string;
	status: number;
}

export interface PaginationParams {
	page?: number;
	page_size?: number;
}
