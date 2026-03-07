import type {
	CollectionResponse,
	CollectionDetailResponse,
	CollectionListResponse,
	CollectionTrackResponse,
	CollectionCreate,
	CollectionUpdate,
	AddTrackToCollection,
	UpdateTrackPosition,
	ReorderTracksRequest,
	CollectionListParams
} from '$lib/types';
import { get, post, patch, put, del } from './client';

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

export async function listCollections(
	params?: CollectionListParams,
	options?: { requireAuth?: boolean }
): Promise<CollectionListResponse> {
	const query = params ? buildQueryString(params) : '';
	return get<CollectionListResponse>(`/collections${query}`, options);
}

export async function createCollection(data: CollectionCreate): Promise<CollectionResponse> {
	return post<CollectionResponse>('/collections', data, { requireAuth: true });
}

export async function getCollection(collectionId: number): Promise<CollectionDetailResponse> {
	return get<CollectionDetailResponse>(`/collections/${collectionId}`);
}

export async function getCollectionBySlug(slug: string): Promise<CollectionDetailResponse> {
	return get<CollectionDetailResponse>(`/collections/by-slug/${slug}`);
}

export async function updateCollection(
	collectionId: number,
	data: CollectionUpdate
): Promise<CollectionResponse> {
	return patch<CollectionResponse>(`/collections/${collectionId}`, data, { requireAuth: true });
}

export async function deleteCollection(collectionId: number): Promise<void> {
	return del<void>(`/collections/${collectionId}`, { requireAuth: true });
}

export async function listCollectionTracks(
	collectionId: number
): Promise<CollectionTrackResponse[]> {
	return get<CollectionTrackResponse[]>(`/collections/${collectionId}/tracks`);
}

export async function addTrackToCollection(
	collectionId: number,
	data: AddTrackToCollection
): Promise<CollectionTrackResponse> {
	return post<CollectionTrackResponse>(`/collections/${collectionId}/tracks`, data, {
		requireAuth: true
	});
}

export async function removeTrackFromCollection(
	collectionId: number,
	trackId: number
): Promise<void> {
	return del<void>(`/collections/${collectionId}/tracks/${trackId}`, { requireAuth: true });
}

export async function updateTrackPosition(
	collectionId: number,
	trackId: number,
	data: UpdateTrackPosition
): Promise<CollectionTrackResponse> {
	return patch<CollectionTrackResponse>(`/collections/${collectionId}/tracks/${trackId}`, data, {
		requireAuth: true
	});
}

export async function reorderTracks(
	collectionId: number,
	data: ReorderTracksRequest
): Promise<CollectionTrackResponse[]> {
	return put<CollectionTrackResponse[]>(`/collections/${collectionId}/tracks/reorder`, data, {
		requireAuth: true
	});
}
