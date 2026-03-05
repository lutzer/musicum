from datetime import datetime

from pydantic import BaseModel, ConfigDict, Field

from backend.schemas.track import TrackResponse


class CollectionCreate(BaseModel):
    name: str = Field(..., max_length=255)
    description: str | None = None
    is_public: bool = False


class CollectionUpdate(BaseModel):
    name: str | None = Field(None, max_length=255)
    description: str | None = None
    is_public: bool | None = None


class CollectionResponse(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    id: int
    name: str
    description: str | None
    user_id: int
    is_public: bool
    created_at: datetime
    updated_at: datetime


class CollectionTrackResponse(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    id: int
    collection_id: int
    track_id: int
    position: int
    added_at: datetime
    track: TrackResponse


class CollectionDetailResponse(CollectionResponse):
    tracks: list[CollectionTrackResponse] = []


class CollectionListResponse(BaseModel):
    items: list[CollectionResponse]
    total: int
    page: int
    page_size: int


class AddTrackToCollection(BaseModel):
    track_id: int
    position: int | None = None


class UpdateTrackPosition(BaseModel):
    position: int = Field(..., ge=0)


class ReorderTracksRequest(BaseModel):
    track_ids: list[int]
