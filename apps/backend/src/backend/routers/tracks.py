import os
import uuid
from pathlib import Path

from fastapi import (
    APIRouter,
    BackgroundTasks,
    Depends,
    File,
    Form,
    HTTPException,
    Query,
    UploadFile,
    status,
)
from fastapi.responses import FileResponse
from fastapi.security import OAuth2PasswordBearer
from jose import JWTError, jwt
from sqlalchemy.orm import Session

from backend.config import settings
from backend.database import get_db
from backend.models.track import AttachmentType, ProcessingState, Track, TrackAttachment
from backend.models.user import User, UserRole
from backend.schemas.track import (
    AttachmentCreate,
    AttachmentResponse,
    AttachmentUpdate,
    ReorderAttachmentsRequest,
    TrackCreate,
    TrackDetailResponse,
    TrackListResponse,
    TrackResponse,
    TrackUpdate,
)
from backend.services.attachment_service import (
    create_attachment,
    delete_attachment,
    get_attachment_by_id,
    get_attachments,
    reorder_attachments,
    update_attachment,
)
from backend.services.audio_processor import process_track_background
from backend.services.media_processor import process_attachment_background
from backend.services.track_service import (
    create_track,
    delete_track,
    generate_unique_track_slug,
    get_track_by_id,
    get_track_by_slug,
    get_track_with_details,
    get_tracks,
    update_track,
    write_track_metadata,
)

router = APIRouter(prefix="/tracks", tags=["tracks"])

oauth2_scheme = OAuth2PasswordBearer(tokenUrl="/auth/login", auto_error=False)

ALLOWED_AUDIO_TYPES = {"audio/mpeg", "audio/wav", "audio/ogg", "audio/flac"}
ALLOWED_IMAGE_TYPES = {"image/jpeg", "image/png", "image/gif", "image/webp"}
ALLOWED_VIDEO_TYPES = {"video/mp4", "video/webm", "video/ogg"}


def get_optional_current_user(
    token: str | None = Depends(oauth2_scheme), db: Session = Depends(get_db)
) -> User | None:
    if token is None:
        return None
    try:
        payload = jwt.decode(
            token, settings.SECRET_KEY, algorithms=[settings.ALGORITHM]
        )
        email: str | None = payload.get("sub")
        if email is None:
            return None
    except JWTError:
        return None
    return db.query(User).filter(User.email == email).first()


def get_required_current_user(
    user: User | None = Depends(get_optional_current_user),
) -> User:
    if user is None:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Authentication required",
            headers={"WWW-Authenticate": "Bearer"},
        )
    return user


def check_track_permission(track: Track, user: User | None, action: str) -> None:
    if user and user.role == UserRole.ADMIN:
        return
    if not user or track.user_id != user.id:
        raise HTTPException(
            status_code=status.HTTP_403_FORBIDDEN,
            detail=f"Not authorized to {action} this track",
        )


def check_track_visibility(track: Track, user: User | None) -> None:
    if track.is_public:
        return
    if user and (user.role == UserRole.ADMIN or track.user_id == user.id):
        return
    raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Track not found")


def save_uploaded_file(file: UploadFile, upload_dir: str) -> tuple[str, str, int]:
    Path(upload_dir).mkdir(parents=True, exist_ok=True)
    filename = file.filename or "unnamed"
    file_path = os.path.join(upload_dir, filename)
    content = file.file.read()
    file_size = len(content)
    with open(file_path, "wb") as f:
        f.write(content)
    return file_path, filename, file_size


def save_attachment_file(file: UploadFile, upload_dir: str) -> tuple[str, str, int]:
    """Save attachment file with UUID filename, preserving original extension."""
    Path(upload_dir).mkdir(parents=True, exist_ok=True)
    original_filename = file.filename or "unnamed"
    ext = Path(original_filename).suffix
    uuid_filename = f"{uuid.uuid4()}{ext}"
    file_path = os.path.join(upload_dir, uuid_filename)
    content = file.file.read()
    file_size = len(content)
    with open(file_path, "wb") as f:
        f.write(content)
    return file_path, original_filename, file_size


@router.get("", response_model=TrackListResponse)
def list_tracks(
    user_id: int | None = Query(None, description="Filter by user ID"),
    tag: str | None = Query(None, description="Filter by tag"),
    page: int = Query(1, ge=1, description="Page number"),
    page_size: int = Query(20, ge=1, le=100, description="Items per page"),
    current_user: User | None = Depends(get_optional_current_user),
    db: Session = Depends(get_db),
) -> dict:
    tracks, total = get_tracks(
        db, user=current_user, user_id=user_id, tag=tag, page=page, page_size=page_size
    )
    return {
        "items": tracks,
        "total": total,
        "page": page,
        "page_size": page_size,
    }


@router.post("", response_model=TrackResponse, status_code=status.HTTP_201_CREATED)
def create_track_endpoint(
    background_tasks: BackgroundTasks,
    file: UploadFile = File(...),
    title: str = Form(...),
    description: str | None = Form(None),
    is_public: bool = Form(False),
    tags: str | None = Form(None),
    current_user: User = Depends(get_required_current_user),
    db: Session = Depends(get_db),
) -> Track:
    if file.content_type not in ALLOWED_AUDIO_TYPES:
        allowed = ", ".join(ALLOWED_AUDIO_TYPES)
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail=f"Invalid file type. Allowed types: {allowed}",
        )
    
    slug = generate_unique_track_slug(db, title)

    track_dir = os.path.join(settings.UPLOAD_DIR_TRACKS, slug)
    Path(track_dir).mkdir(parents=True, exist_ok=True)

    file_path, original_filename, file_size = save_uploaded_file(file, track_dir)

    track_data = TrackCreate(
        title=title, description=description, is_public=is_public, tags=tags
    )
    track = create_track(
        db,
        track_data,
        slug=slug,
        source_path=file_path,
        original_filename=original_filename,
        file_size=file_size,
        mime_type=file.content_type or "audio/mpeg",
        user_id=current_user.id,
        processing_status=ProcessingState.PROCESSING,
    )

  

    write_track_metadata(track_dir, track, original_filename)

    converted_path = os.path.join(settings.UPLOAD_DIR_CONVERTED, f"{slug}.mp3")
    background_tasks.add_task(
        process_track_background,
        track_id=track.id,
        input_path=file_path,
        output_path=converted_path,
    )

    return track


@router.get("/{track_id}", response_model=TrackDetailResponse)
def get_track_endpoint(
    track_id: int,
    current_user: User | None = Depends(get_optional_current_user),
    db: Session = Depends(get_db),
) -> Track:
    track = get_track_with_details(db, track_id)
    if not track:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND, detail="Track not found"
        )
    check_track_visibility(track, current_user)
    return track


@router.patch("/{track_id}", response_model=TrackResponse)
def update_track_endpoint(
    track_id: int,
    track_data: TrackUpdate,
    current_user: User = Depends(get_required_current_user),
    db: Session = Depends(get_db),
) -> Track:
    track = get_track_by_id(db, track_id)
    if not track:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND, detail="Track not found"
        )
    check_track_permission(track, current_user, "update")
    updated_track = update_track(db, track, track_data)

    # Sync track.json
    track_dir = os.path.dirname(updated_track.source_path)
    attachments = get_attachments(db, track_id)
    write_track_metadata(
        track_dir, updated_track, updated_track.original_filename, attachments
    )

    return updated_track


@router.delete("/{track_id}", status_code=status.HTTP_204_NO_CONTENT)
def delete_track_endpoint(
    track_id: int,
    current_user: User = Depends(get_required_current_user),
    db: Session = Depends(get_db),
) -> None:
    track = get_track_by_id(db, track_id)
    if not track:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND, detail="Track not found"
        )
    check_track_permission(track, current_user, "delete")
    delete_track(db, track)


@router.get("/{track_id}/stream")
def stream_track_endpoint(
    track_id: int,
    current_user: User | None = Depends(get_optional_current_user),
    db: Session = Depends(get_db),
) -> FileResponse:
    track = get_track_by_id(db, track_id)
    if not track:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND, detail="Track not found"
        )
    check_track_visibility(track, current_user)

    if track.processing_status == "ready" and track.converted_path:
        if os.path.exists(track.converted_path):
            return FileResponse(
                track.converted_path,
                media_type="audio/mpeg",
                filename=f"{track.slug}.mp3",
            )

    if os.path.exists(track.source_path):
        return FileResponse(
            track.source_path,
            media_type=track.mime_type,
            filename=track.original_filename,
        )

    raise HTTPException(
        status_code=status.HTTP_404_NOT_FOUND, detail="Audio file not found"
    )


@router.get("/by-slug/{slug}", response_model=TrackDetailResponse)
def get_track_by_slug_endpoint(
    slug: str,
    current_user: User | None = Depends(get_optional_current_user),
    db: Session = Depends(get_db),
) -> Track:
    track = get_track_by_slug(db, slug)
    if not track:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND, detail="Track not found"
        )
    check_track_visibility(track, current_user)
    return track


@router.get("/{track_id}/attachments", response_model=list[AttachmentResponse])
def list_attachments_endpoint(
    track_id: int,
    type: AttachmentType | None = Query(None, description="Filter by attachment type"),
    current_user: User | None = Depends(get_optional_current_user),
    db: Session = Depends(get_db),
) -> list[TrackAttachment]:
    track = get_track_by_id(db, track_id)
    if not track:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND, detail="Track not found"
        )
    check_track_visibility(track, current_user)
    return get_attachments(db, track_id, attachment_type=type)


@router.post(
    "/{track_id}/attachments",
    response_model=AttachmentResponse,
    status_code=status.HTTP_201_CREATED,
)
def create_attachment_endpoint(
    track_id: int,
    background_tasks: BackgroundTasks,
    type: AttachmentType = Form(...),
    content: str | None = Form(None),
    caption: str | None = Form(None),
    file: UploadFile | None = File(None),
    current_user: User = Depends(get_required_current_user),
    db: Session = Depends(get_db),
) -> TrackAttachment:
    track = get_track_by_id(db, track_id)
    if not track:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND, detail="Track not found"
        )
    check_track_permission(track, current_user, "modify")

    path = None
    original_filename = None
    processing_status = ProcessingState.READY

    if type == AttachmentType.NOTE:
        if not content:
            raise HTTPException(
                status_code=status.HTTP_400_BAD_REQUEST,
                detail="Content is required for notes",
            )
    elif type == AttachmentType.IMAGE:
        if not file:
            raise HTTPException(
                status_code=status.HTTP_400_BAD_REQUEST,
                detail="File is required for images",
            )
        if file.content_type not in ALLOWED_IMAGE_TYPES:
            allowed = ", ".join(ALLOWED_IMAGE_TYPES)
            raise HTTPException(
                status_code=status.HTTP_400_BAD_REQUEST,
                detail=f"Invalid file type. Allowed types: {allowed}",
            )
        path, original_filename, _ = save_attachment_file(
            file, settings.UPLOAD_DIR_ATTACHMENTS
        )
        processing_status = ProcessingState.PROCESSING
    elif type == AttachmentType.VIDEO:
        if not file:
            raise HTTPException(
                status_code=status.HTTP_400_BAD_REQUEST,
                detail="File is required for videos",
            )
        if file.content_type not in ALLOWED_VIDEO_TYPES:
            allowed = ", ".join(ALLOWED_VIDEO_TYPES)
            raise HTTPException(
                status_code=status.HTTP_400_BAD_REQUEST,
                detail=f"Invalid file type. Allowed types: {allowed}",
            )
        path, original_filename, _ = save_attachment_file(
            file, settings.UPLOAD_DIR_ATTACHMENTS
        )
        processing_status = ProcessingState.PROCESSING

    attachment_data = AttachmentCreate(type=type, content=content, caption=caption)
    attachment = create_attachment(
        db,
        track_id,
        attachment_data,
        path=path,
        original_filename=original_filename,
        processing_status=processing_status,
    )

    if processing_status == ProcessingState.PROCESSING and path:
        processed_dir = os.path.join(settings.UPLOAD_DIR_ATTACHMENTS, "processed")
        if type == AttachmentType.IMAGE:
            processed_path = os.path.join(processed_dir, f"{attachment.id}.jpg")
        else:
            processed_path = os.path.join(processed_dir, f"{attachment.id}.mp4")

        background_tasks.add_task(
            process_attachment_background,
            attachment_id=attachment.id,
            input_path=path,
            output_path=processed_path,
            attachment_type=type.value,
        )

    # Sync track.json
    track_dir = os.path.dirname(track.source_path)
    attachments = get_attachments(db, track_id)
    write_track_metadata(track_dir, track, track.original_filename, attachments)

    return attachment


@router.patch(
    "/{track_id}/attachments/{attachment_id}", response_model=AttachmentResponse
)
def update_attachment_endpoint(
    track_id: int,
    attachment_id: int,
    attachment_data: AttachmentUpdate,
    current_user: User = Depends(get_required_current_user),
    db: Session = Depends(get_db),
) -> TrackAttachment:
    track = get_track_by_id(db, track_id)
    if not track:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND, detail="Track not found"
        )
    check_track_permission(track, current_user, "modify")

    attachment = get_attachment_by_id(db, attachment_id)
    if not attachment or attachment.track_id != track_id:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND, detail="Attachment not found"
        )

    if attachment.type != AttachmentType.NOTE and attachment_data.content is not None:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Content can only be updated for note attachments",
        )

    attachment = update_attachment(db, attachment, attachment_data)

    track_dir = os.path.dirname(track.source_path)
    attachments = get_attachments(db, track_id)
    write_track_metadata(track_dir, track, track.original_filename, attachments)

    return attachment


@router.delete(
    "/{track_id}/attachments/{attachment_id}", status_code=status.HTTP_204_NO_CONTENT
)
def delete_attachment_endpoint(
    track_id: int,
    attachment_id: int,
    current_user: User = Depends(get_required_current_user),
    db: Session = Depends(get_db),
) -> None:
    track = get_track_by_id(db, track_id)
    if not track:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND, detail="Track not found"
        )
    check_track_permission(track, current_user, "modify")

    attachment = get_attachment_by_id(db, attachment_id)
    if not attachment or attachment.track_id != track_id:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND, detail="Attachment not found"
        )

    delete_attachment(db, attachment)

    # Sync track.json
    track_dir = os.path.dirname(track.source_path)
    attachments = get_attachments(db, track_id)
    write_track_metadata(track_dir, track, track.original_filename, attachments)


@router.get("/{track_id}/attachments/{attachment_id}/file")
def get_attachment_file_endpoint(
    track_id: int,
    attachment_id: int,
    current_user: User | None = Depends(get_optional_current_user),
    db: Session = Depends(get_db),
) -> FileResponse:
    track = get_track_by_id(db, track_id)
    if not track:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND, detail="Track not found"
        )
    check_track_visibility(track, current_user)

    attachment = get_attachment_by_id(db, attachment_id)
    if not attachment or attachment.track_id != track_id:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND, detail="Attachment not found"
        )

    if attachment.type == AttachmentType.NOTE:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Notes do not have files",
        )

    if attachment.processing_status == "ready" and attachment.processed_path:
        if os.path.exists(attachment.processed_path):
            media_type = (
                "image/jpeg" if attachment.type == AttachmentType.IMAGE else "video/mp4"
            )
            ext = "jpg" if attachment.type == AttachmentType.IMAGE else "mp4"
            return FileResponse(
                attachment.processed_path,
                media_type=media_type,
                filename=f"{attachment.id}.{ext}",
            )

    if attachment.path and os.path.exists(attachment.path):
        media_type = "application/octet-stream"
        if attachment.type == AttachmentType.IMAGE:
            media_type = "image/jpeg"
        elif attachment.type == AttachmentType.VIDEO:
            media_type = "video/mp4"
        return FileResponse(
            attachment.path,
            media_type=media_type,
            filename=attachment.original_filename or f"attachment_{attachment.id}",
        )

    raise HTTPException(
        status_code=status.HTTP_404_NOT_FOUND, detail="Attachment file not found"
    )


@router.put("/{track_id}/attachments/reorder", response_model=list[AttachmentResponse])
def reorder_attachments_endpoint(
    track_id: int,
    request: ReorderAttachmentsRequest,
    current_user: User = Depends(get_required_current_user),
    db: Session = Depends(get_db),
) -> list[TrackAttachment]:
    track = get_track_by_id(db, track_id)
    if not track:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND, detail="Track not found"
        )
    check_track_permission(track, current_user, "modify")

    return reorder_attachments(db, track_id, request.attachment_ids)
