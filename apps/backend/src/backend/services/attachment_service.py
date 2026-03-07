from sqlalchemy.orm import Session

from backend.models.track import AttachmentType, TrackAttachment
from backend.schemas.track import AttachmentCreate, AttachmentUpdate
from backend.utils import delete_file


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
    return query.order_by(
        TrackAttachment.position, TrackAttachment.created_at.desc()
    ).all()


def get_next_attachment_position(db: Session, track_id: int) -> int:
    """Get the next position for a new attachment."""
    max_position = (
        db.query(TrackAttachment).filter(TrackAttachment.track_id == track_id).count()
    )
    return max_position


def reorder_attachments(
    db: Session, track_id: int, attachment_ids: list[int]
) -> list[TrackAttachment]:
    """Reorder attachments by setting their positions."""
    attachments = (
        db.query(TrackAttachment).filter(TrackAttachment.track_id == track_id).all()
    )
    attachment_map = {a.id: a for a in attachments}

    for position, attachment_id in enumerate(attachment_ids):
        if attachment_id in attachment_map:
            attachment_map[attachment_id].position = position

    db.commit()
    return get_attachments(db, track_id)


def create_attachment(
    db: Session,
    track_id: int,
    attachment_data: AttachmentCreate,
    path: str | None = None,
    original_filename: str | None = None,
    processing_status: str = "ready",
) -> TrackAttachment:
    position = get_next_attachment_position(db, track_id)
    attachment = TrackAttachment(
        track_id=track_id,
        type=attachment_data.type,
        content=attachment_data.content,
        path=path,
        original_filename=original_filename,
        caption=attachment_data.caption,
        position=position,
        processing_status=processing_status,
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
    if attachment.path:
        delete_file(attachment.path)
    if attachment.processed_path:
        delete_file(attachment.processed_path)