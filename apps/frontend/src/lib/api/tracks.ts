import type {
	TrackResponse,
	TrackDetailResponse,
	TrackListResponse,
	TrackCreate,
	TrackUpdate,
	AttachmentResponse,
	AttachmentCreate,
	AttachmentUpdate,
	TrackListParams
} from '$lib/types';
import { AttachmentType } from '$lib/types';
import { get, patch, del, postFormData } from './client';

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

export async function listTracks(params?: TrackListParams): Promise<TrackListResponse> {
	const query = params ? buildQueryString(params) : '';
	return get<TrackListResponse>(`/tracks${query}`);
}

export async function createTrack(file: File, data: TrackCreate): Promise<TrackResponse> {
	const formData = new FormData();
	formData.append('file', file);
	formData.append('title', data.title);
	if (data.description !== undefined && data.description !== null) {
		formData.append('description', data.description);
	}
	if (data.is_public !== undefined) {
		formData.append('is_public', String(data.is_public));
	}
	if (data.tags !== undefined && data.tags !== null) {
		formData.append('tags', data.tags);
	}
	return postFormData<TrackResponse>('/tracks', formData, { requireAuth: true });
}

export async function getTrack(trackId: number): Promise<TrackDetailResponse> {
	return get<TrackDetailResponse>(`/tracks/${trackId}`);
}

export async function getTrackBySlug(slug: string): Promise<TrackDetailResponse> {
	return get<TrackDetailResponse>(`/tracks/by-slug/${slug}`);
}

export async function updateTrack(trackId: number, data: TrackUpdate): Promise<TrackResponse> {
	return patch<TrackResponse>(`/tracks/${trackId}`, data, { requireAuth: true });
}

export async function deleteTrack(trackId: number): Promise<void> {
	return del<void>(`/tracks/${trackId}`, { requireAuth: true });
}

export async function listAttachments(
	trackId: number,
	type?: AttachmentType
): Promise<AttachmentResponse[]> {
	const query = type ? `?type=${type}` : '';
	return get<AttachmentResponse[]>(`/tracks/${trackId}/attachments${query}`);
}

export async function createAttachment(
	trackId: number,
	data: AttachmentCreate,
	file?: File
): Promise<AttachmentResponse> {
	const formData = new FormData();
	formData.append('type', data.type);
	if (data.content !== undefined && data.content !== null) {
		formData.append('content', data.content);
	}
	if (data.caption !== undefined && data.caption !== null) {
		formData.append('caption', data.caption);
	}
	if (file) {
		formData.append('file', file);
	}
	return postFormData<AttachmentResponse>(`/tracks/${trackId}/attachments`, formData, {
		requireAuth: true
	});
}

export async function updateAttachment(
	trackId: number,
	attachmentId: number,
	data: AttachmentUpdate
): Promise<AttachmentResponse> {
	return patch<AttachmentResponse>(`/tracks/${trackId}/attachments/${attachmentId}`, data, {
		requireAuth: true
	});
}

export async function deleteAttachment(trackId: number, attachmentId: number): Promise<void> {
	return del<void>(`/tracks/${trackId}/attachments/${attachmentId}`, { requireAuth: true });
}
