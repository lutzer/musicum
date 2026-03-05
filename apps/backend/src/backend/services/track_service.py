from sqlalchemy.orm import Session, joinedload

from backend.models.track import AttachmentType, Track, TrackAttachment
from backend.models.user import User, UserRole
from backend.schemas.track import (
    AttachmentCreate,
    AttachmentUpdate,
    TrackCreate,
    TrackUpdate,
)


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
    source_path: str,
    original_filename: str,
    file_size: int,
    mime_type: str,
    user_id: int | None = None,
    duration_seconds: float | None = None,
) -> Track:
    track = Track(
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
    )
    db.add(track)
    db.commit()
    db.refresh(track)
    return track


def update_track(db: Session, track: Track, track_data: TrackUpdate) -> Track:
    update_data = track_data.model_dump(exclude_unset=True)
    for field, value in update_data.items():
        setattr(track, field, value)
    db.commit()
    db.refresh(track)
    return track


def delete_track(db: Session, track: Track) -> None:
    db.delete(track)
    db.commit()


def get_attachment_by_id(db: Session, attachment_id: int) -> TrackAttachment | None:
    return db.query(TrackAttachment).filter(TrackAttachment.id == attachment_id).first()


def get_attachments(
    db: Session,
    track_id: int,
    attachment_type: AttachmentType | None = None,
) -> list[TrackAttachment]:
    query = db.query(TrackAttachment).filter(TrackAttachment.track_id == track_id)
    if attachment_type is not None:
        query = query.filter(TrackAttachment.type == attachment_type)
    return query.order_by(TrackAttachment.created_at.desc()).all()


def create_attachment(
    db: Session,
    track_id: int,
    attachment_data: AttachmentCreate,
    path: str | None = None,
    original_filename: str | None = None,
) -> TrackAttachment:
    attachment = TrackAttachment(
        track_id=track_id,
        type=attachment_data.type,
        content=attachment_data.content,
        path=path,
        original_filename=original_filename,
        caption=attachment_data.caption,
    )
    db.add(attachment)
    db.commit()
    db.refresh(attachment)
    return attachment


def update_attachment(
    db: Session, attachment: TrackAttachment, attachment_data: AttachmentUpdate
) -> TrackAttachment:
    update_data = attachment_data.model_dump(exclude_unset=True)
    for field, value in update_data.items():
        setattr(attachment, field, value)
    db.commit()
    db.refresh(attachment)
    return attachment


def delete_attachment(db: Session, attachment: TrackAttachment) -> None:
    db.delete(attachment)
    db.commit()
