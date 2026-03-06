from datetime import datetime

from pydantic import BaseModel, ConfigDict, Field, computed_field

from backend.models.track import AttachmentType


class TrackCreate(BaseModel):
    title: str = Field(..., max_length=255)
    description: str | None = None
    is_public: bool = False
    tags: str | None = Field(None, max_length=500)


class TrackUpdate(BaseModel):
    title: str | None = Field(None, max_length=255)
    description: str | None = None
    is_public: bool | None = None
    tags: str | None = Field(None, max_length=500)


class TrackResponse(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    id: int
    slug: str
    title: str
    description: str | None
    original_filename: str
    file_size: int
    mime_type: str
    duration_seconds: float | None
    user_id: int | None
    is_public: bool
    tags: str | None
    processing_status: str
    converted_path: str | None
    created_at: datetime
    updated_at: datetime

    @computed_field
    @property
    def stream_url(self) -> str:
        return f"/tracks/{self.id}/stream"


class AttachmentResponse(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    id: int
    track_id: int
    type: AttachmentType
    content: str | None
    path: str | None
    original_filename: str | None
    caption: str | None
    created_at: datetime
    updated_at: datetime


class TrackDetailResponse(TrackResponse):
    attachments: list[AttachmentResponse] = []


class TrackListResponse(BaseModel):
    items: list[TrackResponse]
    total: int
    page: int
    page_size: int


class AttachmentCreate(BaseModel):
    type: AttachmentType
    content: str | None = None
    caption: str | None = Field(None, max_length=500)


class AttachmentUpdate(BaseModel):
    content: str | None = None
    caption: str | None = Field(None, max_length=500)
