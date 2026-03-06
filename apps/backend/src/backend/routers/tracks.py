import json
import os
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
from backend.models.track import AttachmentType, Track, TrackAttachment
from backend.models.user import User, UserRole
from backend.schemas.track import (
    AttachmentCreate,
    AttachmentResponse,
    AttachmentUpdate,
    TrackCreate,
    TrackDetailResponse,
    TrackListResponse,
    TrackResponse,
    TrackUpdate,
)
from backend.services.audio_processor import process_track_background
from backend.services.track_service import (
    create_attachment,
    create_track,
    delete_attachment,
    delete_track,
    generate_unique_slug,
    get_attachment_by_id,
    get_attachments,
    get_track_by_id,
    get_track_by_slug,
    get_track_with_details,
    get_tracks,
    update_attachment,
    update_track,
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


def write_track_metadata(track_dir: str, track: Track, original_filename: str) -> None:
    """Write track.json metadata file."""
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
    }
    metadata_path = os.path.join(track_dir, "track.json")
    with open(metadata_path, "w") as f:
        json.dump(metadata, f, indent=2)


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

    slug = generate_unique_slug(db, title)

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
        processing_status="processing",
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
    return update_track(db, track, track_data)


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
    if os.path.exists(track.source_path):
        os.remove(track.source_path)
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
        path, original_filename, _ = save_uploaded_file(
            file, settings.UPLOAD_DIR_ATTACHMENTS
        )
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
        path, original_filename, _ = save_uploaded_file(
            file, settings.UPLOAD_DIR_ATTACHMENTS
        )

    attachment_data = AttachmentCreate(type=type, content=content, caption=caption)
    return create_attachment(
        db, track_id, attachment_data, path=path, original_filename=original_filename
    )


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

    if attachment.type != AttachmentType.NOTE:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Only note attachments can be updated",
        )

    return update_attachment(db, attachment, attachment_data)


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

    if attachment.path and os.path.exists(attachment.path):
        os.remove(attachment.path)

    delete_attachment(db, attachment)
