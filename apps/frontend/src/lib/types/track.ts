export enum AttachmentType {
	NOTE = 'note',
	IMAGE = 'image',
	VIDEO = 'video'
}

export interface TrackResponse {
	id: number;
	slug: string;
	title: string;
	description: string | null;
	original_filename: string;
	file_size: number;
	mime_type: string;
	duration_seconds: number | null;
	user_id: number | null;
	is_public: boolean;
	tags: string | null;
	processing_status: string;
	converted_path: string | null;
	stream_url: string;
	created_at: string;
	updated_at: string;
}

export interface AttachmentResponse {
	id: number;
	track_id: number;
	type: AttachmentType;
	content: string | null;
	path: string | null;
	original_filename: string | null;
	caption: string | null;
	position: number;
	processed_path: string | null;
	processing_status: string;
	file_url: string | null;
	created_at: string;
	updated_at: string;
}

export interface TrackDetailResponse extends TrackResponse {
	attachments: AttachmentResponse[];
}

export interface TrackListResponse {
	items: TrackResponse[];
	total: number;
	page: number;
	page_size: number;
}

export interface TrackCreate {
	title: string;
	description?: string | null;
	is_public?: boolean;
	tags?: string | null;
}

export interface TrackUpdate {
	title?: string | null;
	description?: string | null;
	is_public?: boolean | null;
	tags?: string | null;
}

export interface AttachmentCreate {
	type: AttachmentType;
	content?: string | null;
	caption?: string | null;
}

export interface AttachmentUpdate {
	content?: string | null;
	caption?: string | null;
}

export interface TrackListParams {
	user_id?: number;
	tag?: string;
	page?: number;
	page_size?: number;
}
