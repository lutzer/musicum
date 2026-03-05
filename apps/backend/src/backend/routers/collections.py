from fastapi import APIRouter, Depends, HTTPException, Query, status
from sqlalchemy.orm import Session

from backend.database import get_db
from backend.models.collection import Collection, CollectionTrack
from backend.models.user import User, UserRole
from backend.routers.tracks import get_optional_current_user, get_required_current_user
from backend.schemas.collection import (
    AddTrackToCollection,
    CollectionCreate,
    CollectionDetailResponse,
    CollectionListResponse,
    CollectionResponse,
    CollectionTrackResponse,
    CollectionUpdate,
    ReorderTracksRequest,
    UpdateTrackPosition,
)
from backend.services.collection_service import (
    add_track_to_collection,
    create_collection,
    delete_collection,
    get_collection_by_id,
    get_collection_track,
    get_collection_tracks,
    get_collection_with_tracks,
    get_collections,
    remove_track_from_collection,
    reorder_tracks,
    update_collection,
    update_track_position,
)
from backend.services.track_service import get_track_by_id

router = APIRouter(prefix="/collections", tags=["collections"])


def check_collection_permission(
    collection: Collection, user: User | None, action: str
) -> None:
    if user and user.role == UserRole.ADMIN:
        return
    if not user or collection.user_id != user.id:
        raise HTTPException(
            status_code=status.HTTP_403_FORBIDDEN,
            detail=f"Not authorized to {action} this collection",
        )


def check_collection_visibility(collection: Collection, user: User | None) -> None:
    if collection.is_public:
        return
    if user and (user.role == UserRole.ADMIN or collection.user_id == user.id):
        return
    raise HTTPException(
        status_code=status.HTTP_404_NOT_FOUND, detail="Collection not found"
    )


@router.get("", response_model=CollectionListResponse)
def list_collections(
    user_id: int | None = Query(None, description="Filter by user ID"),
    page: int = Query(1, ge=1, description="Page number"),
    page_size: int = Query(20, ge=1, le=100, description="Items per page"),
    current_user: User | None = Depends(get_optional_current_user),
    db: Session = Depends(get_db),
) -> dict:
    collections, total = get_collections(
        db, user=current_user, user_id=user_id, page=page, page_size=page_size
    )
    return {
        "items": collections,
        "total": total,
        "page": page,
        "page_size": page_size,
    }


@router.post("", response_model=CollectionResponse, status_code=status.HTTP_201_CREATED)
def create_collection_endpoint(
    collection_data: CollectionCreate,
    current_user: User = Depends(get_required_current_user),
    db: Session = Depends(get_db),
) -> Collection:
    return create_collection(db, collection_data, user_id=current_user.id)


@router.get("/{collection_id}", response_model=CollectionDetailResponse)
def get_collection_endpoint(
    collection_id: int,
    current_user: User | None = Depends(get_optional_current_user),
    db: Session = Depends(get_db),
) -> dict:
    collection = get_collection_with_tracks(db, collection_id)
    if not collection:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND, detail="Collection not found"
        )
    check_collection_visibility(collection, current_user)

    # Sort tracks by position and transform to expected format
    sorted_tracks = sorted(collection.collection_tracks, key=lambda ct: ct.position)

    return {
        "id": collection.id,
        "name": collection.name,
        "description": collection.description,
        "user_id": collection.user_id,
        "is_public": collection.is_public,
        "created_at": collection.created_at,
        "updated_at": collection.updated_at,
        "tracks": sorted_tracks,
    }


@router.patch("/{collection_id}", response_model=CollectionResponse)
def update_collection_endpoint(
    collection_id: int,
    collection_data: CollectionUpdate,
    current_user: User = Depends(get_required_current_user),
    db: Session = Depends(get_db),
) -> Collection:
    collection = get_collection_by_id(db, collection_id)
    if not collection:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND, detail="Collection not found"
        )
    check_collection_permission(collection, current_user, "update")
    return update_collection(db, collection, collection_data)


@router.delete("/{collection_id}", status_code=status.HTTP_204_NO_CONTENT)
def delete_collection_endpoint(
    collection_id: int,
    current_user: User = Depends(get_required_current_user),
    db: Session = Depends(get_db),
) -> None:
    collection = get_collection_by_id(db, collection_id)
    if not collection:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND, detail="Collection not found"
        )
    check_collection_permission(collection, current_user, "delete")
    delete_collection(db, collection)


@router.get("/{collection_id}/tracks", response_model=list[CollectionTrackResponse])
def list_collection_tracks_endpoint(
    collection_id: int,
    current_user: User | None = Depends(get_optional_current_user),
    db: Session = Depends(get_db),
) -> list[CollectionTrack]:
    collection = get_collection_by_id(db, collection_id)
    if not collection:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND, detail="Collection not found"
        )
    check_collection_visibility(collection, current_user)
    return get_collection_tracks(db, collection_id)


@router.post(
    "/{collection_id}/tracks",
    response_model=CollectionTrackResponse,
    status_code=status.HTTP_201_CREATED,
)
def add_track_to_collection_endpoint(
    collection_id: int,
    track_data: AddTrackToCollection,
    current_user: User = Depends(get_required_current_user),
    db: Session = Depends(get_db),
) -> CollectionTrack:
    collection = get_collection_by_id(db, collection_id)
    if not collection:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND, detail="Collection not found"
        )
    check_collection_permission(collection, current_user, "modify")

    # Check if track exists
    track = get_track_by_id(db, track_data.track_id)
    if not track:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND, detail="Track not found"
        )

    # Check if track is already in the collection
    existing = get_collection_track(db, collection_id, track_data.track_id)
    if existing:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Track is already in this collection",
        )

    collection_track = add_track_to_collection(
        db, collection_id, track_data.track_id, track_data.position
    )
    # Reload with track relationship
    db.refresh(collection_track)
    return collection_track


@router.delete(
    "/{collection_id}/tracks/{track_id}", status_code=status.HTTP_204_NO_CONTENT
)
def remove_track_from_collection_endpoint(
    collection_id: int,
    track_id: int,
    current_user: User = Depends(get_required_current_user),
    db: Session = Depends(get_db),
) -> None:
    collection = get_collection_by_id(db, collection_id)
    if not collection:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND, detail="Collection not found"
        )
    check_collection_permission(collection, current_user, "modify")

    collection_track = get_collection_track(db, collection_id, track_id)
    if not collection_track:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Track not found in collection",
        )

    remove_track_from_collection(db, collection_track)


@router.patch(
    "/{collection_id}/tracks/{track_id}", response_model=CollectionTrackResponse
)
def update_track_position_endpoint(
    collection_id: int,
    track_id: int,
    position_data: UpdateTrackPosition,
    current_user: User = Depends(get_required_current_user),
    db: Session = Depends(get_db),
) -> CollectionTrack:
    collection = get_collection_by_id(db, collection_id)
    if not collection:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND, detail="Collection not found"
        )
    check_collection_permission(collection, current_user, "modify")

    collection_track = get_collection_track(db, collection_id, track_id)
    if not collection_track:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="Track not found in collection",
        )

    updated = update_track_position(db, collection_track, position_data.position)
    db.refresh(updated)
    return updated


@router.put(
    "/{collection_id}/tracks/reorder", response_model=list[CollectionTrackResponse]
)
def reorder_tracks_endpoint(
    collection_id: int,
    reorder_data: ReorderTracksRequest,
    current_user: User = Depends(get_required_current_user),
    db: Session = Depends(get_db),
) -> list[CollectionTrack]:
    collection = get_collection_by_id(db, collection_id)
    if not collection:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND, detail="Collection not found"
        )
    check_collection_permission(collection, current_user, "modify")

    return reorder_tracks(db, collection_id, reorder_data.track_ids)
