from sqlalchemy import func
from sqlalchemy.orm import Session, joinedload

from backend.models.collection import Collection, CollectionTrack
from backend.models.user import User, UserRole
from backend.schemas.collection import CollectionCreate, CollectionUpdate


def get_collection_by_id(db: Session, collection_id: int) -> Collection | None:
    return db.query(Collection).filter(Collection.id == collection_id).first()


def get_collection_with_tracks(db: Session, collection_id: int) -> Collection | None:
    return (
        db.query(Collection)
        .options(
            joinedload(Collection.collection_tracks).joinedload(CollectionTrack.track)
        )
        .filter(Collection.id == collection_id)
        .first()
    )


def get_collections(
    db: Session,
    user: User | None = None,
    user_id: int | None = None,
    page: int = 1,
    page_size: int = 20,
) -> tuple[list[Collection], int]:
    query = db.query(Collection)

    # Apply visibility filter based on current user
    if user is None:
        # Anonymous users can only see public collections
        query = query.filter(Collection.is_public == True)  # noqa: E712
    elif user.role != UserRole.ADMIN:
        # Non-admin users can see public collections or their own collections
        query = query.filter(
            (Collection.is_public == True) | (Collection.user_id == user.id)  # noqa: E712
        )
    # Admins can see all collections (no filter needed)

    # Filter by specific user's collections
    if user_id is not None:
        query = query.filter(Collection.user_id == user_id)

    total = query.count()
    collections = (
        query.order_by(Collection.created_at.desc())
        .offset((page - 1) * page_size)
        .limit(page_size)
        .all()
    )
    return collections, total


def create_collection(
    db: Session,
    collection_data: CollectionCreate,
    user_id: int,
) -> Collection:
    collection = Collection(
        name=collection_data.name,
        description=collection_data.description,
        user_id=user_id,
        is_public=collection_data.is_public,
    )
    db.add(collection)
    db.commit()
    db.refresh(collection)
    return collection


def update_collection(
    db: Session, collection: Collection, collection_data: CollectionUpdate
) -> Collection:
    update_data = collection_data.model_dump(exclude_unset=True)
    for field, value in update_data.items():
        setattr(collection, field, value)
    db.commit()
    db.refresh(collection)
    return collection


def delete_collection(db: Session, collection: Collection) -> None:
    db.delete(collection)
    db.commit()


def get_collection_track(
    db: Session, collection_id: int, track_id: int
) -> CollectionTrack | None:
    return (
        db.query(CollectionTrack)
        .filter(
            CollectionTrack.collection_id == collection_id,
            CollectionTrack.track_id == track_id,
        )
        .first()
    )


def get_collection_tracks(db: Session, collection_id: int) -> list[CollectionTrack]:
    return (
        db.query(CollectionTrack)
        .options(joinedload(CollectionTrack.track))
        .filter(CollectionTrack.collection_id == collection_id)
        .order_by(CollectionTrack.position)
        .all()
    )


def add_track_to_collection(
    db: Session,
    collection_id: int,
    track_id: int,
    position: int | None = None,
) -> CollectionTrack:
    # If no position specified, add at the end
    if position is None:
        max_position = (
            db.query(func.max(CollectionTrack.position))
            .filter(CollectionTrack.collection_id == collection_id)
            .scalar()
        )
        position = (max_position or 0) + 1

    collection_track = CollectionTrack(
        collection_id=collection_id,
        track_id=track_id,
        position=position,
    )
    db.add(collection_track)
    db.commit()
    db.refresh(collection_track)
    return collection_track


def remove_track_from_collection(
    db: Session, collection_track: CollectionTrack
) -> None:
    db.delete(collection_track)
    db.commit()


def update_track_position(
    db: Session, collection_track: CollectionTrack, position: int
) -> CollectionTrack:
    collection_track.position = position
    db.commit()
    db.refresh(collection_track)
    return collection_track


def reorder_tracks(
    db: Session, collection_id: int, track_ids: list[int]
) -> list[CollectionTrack]:
    # Update positions based on the order of track_ids
    for position, track_id in enumerate(track_ids):
        collection_track = get_collection_track(db, collection_id, track_id)
        if collection_track:
            collection_track.position = position

    db.commit()
    return get_collection_tracks(db, collection_id)
