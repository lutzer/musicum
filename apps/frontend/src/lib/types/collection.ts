import type { TrackResponse } from './track';

export interface CollectionResponse {
	id: number;
	name: string;
	description: string | null;
	user_id: number;
	is_public: boolean;
	created_at: string;
	updated_at: string;
}

export interface CollectionTrackResponse {
	id: number;
	collection_id: number;
	track_id: number;
	position: number;
	added_at: string;
	track: TrackResponse;
}

export interface CollectionDetailResponse extends CollectionResponse {
	tracks: CollectionTrackResponse[];
}

export interface CollectionListResponse {
	items: CollectionResponse[];
	total: number;
	page: number;
	page_size: number;
}

export interface CollectionCreate {
	name: string;
	description?: string | null;
	is_public?: boolean;
}

export interface CollectionUpdate {
	name?: string | null;
	description?: string | null;
	is_public?: boolean | null;
}

export interface AddTrackToCollection {
	track_id: number;
	position?: number | null;
}

export interface UpdateTrackPosition {
	position: number;
}

export interface ReorderTracksRequest {
	track_ids: number[];
}

export interface CollectionListParams {
	user_id?: number;
	page?: number;
	page_size?: number;
}
