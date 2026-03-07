import json
import os
from pathlib import Path
import uuid

from slugify import slugify
from sqlalchemy.orm import Session, joinedload

from backend.models.track import AttachmentType, Track, TrackAttachment
from backend.models.user import User, UserRole
from backend.schemas.track import (
    TrackCreate,
    TrackUpdate,
)
from backend.services.attachment_service import delete_attachment
from backend.utils import delete_dir, delete_file

def generate_unique_slug(
    db: Session, title: str, exclude_track_id: int | None = None
) -> str:
    """Generate a unique slug from the title."""
    base_slug = slugify(title, max_length=200)
    if not base_slug:
        base_slug = "track"

    slug = base_slug
    query = db.query(Track).filter(Track.slug == slug)
    if exclude_track_id is not None:
        query = query.filter(Track.id != exclude_track_id)
    existing = query.first()
    if existing:
        suffix = uuid.uuid4().hex[:4]
        slug = f"{base_slug}-{suffix}"

    return slug


def get_track_by_slug(db: Session, slug: str) -> Track | None:
    """Get a track by its slug."""
    return db.query(Track).filter(Track.slug == slug).first()


def update_track_status(
    db: Session,
    track_id: int,
    status: str,
    converted_path: str | None = None,
) -> Track | None:
    """Update a track's processing status."""
    track = db.query(Track).filter(Track.id == track_id).first()
    if not track:
        return None
    track.processing_status = status
    if converted_path:
        track.converted_path = converted_path
    db.commit()
    db.refresh(track)
    return track


def get_track_by_id(db: Session, track_id: int) -> Track | None:
    return db.query(Track).filter(Track.id == track_id).first()


def get_track_with_details(db: Session, track_id: int) -> Track | None:
    return (
        db.query(Track)
        .options(joinedload(Track.attachments))
        .filter(Track.id == track_id)
        .first()
    )


def get_tracks(
    db: Session,
    user: User | None = None,
    user_id: int | None = None,
    tag: str | None = None,
    page: int = 1,
    page_size: int = 20,
) -> tuple[list[Track], int]:
    query = db.query(Track)

    # Apply visibility filter based on current user
    if user is None:
        # Anonymous users can only see public tracks
        query = query.filter(Track.is_public == True)  # noqa: E712
    elif user.role != UserRole.ADMIN:
        # Non-admin users can see public tracks or their own tracks
        query = query.filter(
            (Track.is_public == True) | (Track.user_id == user.id)  # noqa: E712
        )
    # Admins can see all tracks (no filter needed)

    # Filter by specific user's tracks
    if user_id is not None:
        query = query.filter(Track.user_id == user_id)

    # Filter by tag (case-insensitive substring match in comma-separated tags)
    if tag is not None:
        query = query.filter(Track.tags.ilike(f"%{tag}%"))

    total = query.count()
    tracks = (
        query.order_by(Track.created_at.desc())
        .offset((page - 1) * page_size)
        .limit(page_size)
        .all()
    )
    return tracks, total


def create_track(
    db: Session,
    track_data: TrackCreate,
    slug: str,
    source_path: str,
    original_filename: str,
    file_size: int,
    mime_type: str,
    user_id: int | None = None,
    duration_seconds: float | None = None,
    processing_status: str = "processing",
) -> Track:
    track = Track(
        slug=slug,
        title=track_data.title,
        description=track_data.description,
        source_path=source_path,
        original_filename=original_filename,
        file_size=file_size,
        mime_type=mime_type,
        duration_seconds=duration_seconds,
        user_id=user_id,
        is_public=track_data.is_public,
        tags=track_data.tags,
        processing_status=processing_status,
    )
    db.add(track)
    db.commit()
    db.refresh(track)
    return track


def update_track(db: Session, track: Track, track_data: TrackUpdate) -> Track:
    update_data = track_data.model_dump(exclude_unset=True)
    for field, value in update_data.items():
        setattr(track, field, value)

    # Regenerate slug if title changed
    if "title" in update_data and update_data["title"]:
        track.slug = generate_unique_slug(
            db, update_data["title"], exclude_track_id=track.id
        )

    db.commit()
    db.refresh(track)
    return track


def delete_track(db: Session, track: Track, track_dir: str) -> None:
    db.delete(track)
    db.commit()
    delete_file(track.converted_path)
    delete_dir(track_dir)
    for attachment in track.attachments:
        delete_attachment(db, attachment)

def write_track_metadata(
    track_dir: str,
    track: Track,
    original_filename: str,
    attachments: list[TrackAttachment] | None = None,
) -> None:
    
    if not Path(track_dir).exists():
        raise Exception("Track dir does not exist: " + track_dir)
    
    """Write track.json metadata file."""
    attachments_data = []
    if attachments:
        for att in attachments:
            attachments_data.append(
                {
                    "id": att.id,
                    "type": att.type.value,
                    "caption": att.caption,
                    "position": att.position,
                    "original_filename": att.original_filename,
                }
            )

    metadata = {
        "id": track.id,
        "slug": track.slug,
        "title": track.title,
        "description": track.description,
        "original_filename": original_filename,
        "file_size": track.file_size,
        "mime_type": track.mime_type,
        "duration_seconds": track.duration_seconds,
        "is_public": track.is_public,
        "tags": track.tags,
        "user_id": track.user_id,
        "created_at": track.created_at.isoformat() if track.created_at else None,
        "updated_at": track.updated_at.isoformat() if track.updated_at else None,
        "attachments": attachments_data,
    }
    metadata_path = os.path.join(track_dir, "track.json")
    with open(metadata_path, "w") as f:
        json.dump(metadata, f, indent=2)



