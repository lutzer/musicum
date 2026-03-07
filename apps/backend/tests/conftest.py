from collections.abc import Generator

import pytest
from fastapi.testclient import TestClient
from sqlalchemy import create_engine
from sqlalchemy.orm import Session, sessionmaker
from sqlalchemy.pool import StaticPool

from backend.database import Base, get_db
from backend.main import app
from backend.models.collection import Collection, CollectionTrack
from backend.models.track import AttachmentType, Track, TrackAttachment
from backend.models.user import User, UserRole
from backend.services.auth_service import create_access_token, get_password_hash


@pytest.fixture
def db_session() -> Generator[Session, None, None]:
    engine = create_engine(
        "sqlite:///:memory:",
        connect_args={"check_same_thread": False},
        poolclass=StaticPool,
    )
    TestingSessionLocal = sessionmaker(autocommit=False, autoflush=False, bind=engine)
    Base.metadata.create_all(bind=engine)

    session = TestingSessionLocal()
    try:
        yield session
    finally:
        session.close()
        Base.metadata.drop_all(bind=engine)


@pytest.fixture
def client(db_session: Session) -> Generator[TestClient, None, None]:
    def override_get_db():
        try:
            yield db_session
        finally:
            pass

    app.dependency_overrides[get_db] = override_get_db
    with TestClient(app) as test_client:
        yield test_client
    app.dependency_overrides.clear()


@pytest.fixture
def test_user(db_session: Session) -> User:
    user = User(
        username="testuser",
        email="test@example.com",
        hashed_password=get_password_hash("testpassword"),
        role=UserRole.USER,
    )
    db_session.add(user)
    db_session.commit()
    db_session.refresh(user)
    return user


@pytest.fixture
def admin_user(db_session: Session) -> User:
    user = User(
        username="adminuser",
        email="admin@example.com",
        hashed_password=get_password_hash("adminpassword"),
        role=UserRole.ADMIN,
    )
    db_session.add(user)
    db_session.commit()
    db_session.refresh(user)
    return user


@pytest.fixture
def auth_headers(test_user: User) -> dict:
    token = create_access_token(data={"sub": test_user.email})
    return {"Authorization": f"Bearer {token}"}


@pytest.fixture
def admin_auth_headers(admin_user: User) -> dict:
    token = create_access_token(data={"sub": admin_user.email})
    return {"Authorization": f"Bearer {token}"}


@pytest.fixture
def other_user(db_session: Session) -> User:
    user = User(
        username="otheruser",
        email="other@example.com",
        hashed_password=get_password_hash("otherpassword"),
        role=UserRole.USER,
    )
    db_session.add(user)
    db_session.commit()
    db_session.refresh(user)
    return user


@pytest.fixture
def other_auth_headers(other_user: User) -> dict:
    token = create_access_token(data={"sub": other_user.email})
    return {"Authorization": f"Bearer {token}"}


@pytest.fixture
def public_track(db_session: Session, test_user: User) -> Track:
    track = Track(
        slug="public-track",
        title="Public Track",
        description="A public track",
        source_path="/fake/path/public.mp3",
        original_filename="public.mp3",
        file_size=1024,
        mime_type="audio/mpeg",
        duration_seconds=120.0,
        user_id=test_user.id,
        is_public=True,
        tags="rock,indie",
        processing_status="ready",
    )
    db_session.add(track)
    db_session.commit()
    db_session.refresh(track)
    return track


@pytest.fixture
def private_track(db_session: Session, test_user: User) -> Track:
    track = Track(
        slug="private-track",
        title="Private Track",
        description="A private track",
        source_path="/fake/path/private.mp3",
        original_filename="private.mp3",
        file_size=2048,
        mime_type="audio/mpeg",
        duration_seconds=180.0,
        user_id=test_user.id,
        is_public=False,
        tags="jazz,ambient",
        processing_status="ready",
    )
    db_session.add(track)
    db_session.commit()
    db_session.refresh(track)
    return track


@pytest.fixture
def track_with_attachments(db_session: Session, test_user: User) -> Track:
    track = Track(
        slug="track-with-attachments",
        title="Track With Attachments",
        description="A track with various attachments",
        source_path="/fake/path/attachments.mp3",
        original_filename="attachments.mp3",
        file_size=3072,
        mime_type="audio/mpeg",
        duration_seconds=240.0,
        user_id=test_user.id,
        is_public=True,
        tags="electronic",
        processing_status="ready",
    )
    db_session.add(track)
    db_session.commit()
    db_session.refresh(track)

    note = TrackAttachment(
        track_id=track.id,
        type=AttachmentType.NOTE,
        content="This is a note about the track.",
        caption=None,
        position=0,
        processing_status="ready",
    )
    image = TrackAttachment(
        track_id=track.id,
        type=AttachmentType.IMAGE,
        path="/fake/path/image.jpg",
        original_filename="image.jpg",
        caption="Album cover",
        position=1,
        processing_status="ready",
    )
    db_session.add_all([note, image])
    db_session.commit()
    db_session.refresh(track)
    return track


@pytest.fixture
def public_collection(db_session: Session, test_user: User) -> Collection:
    collection = Collection(
        name="Public Collection",
        description="A public collection",
        user_id=test_user.id,
        is_public=True,
    )
    db_session.add(collection)
    db_session.commit()
    db_session.refresh(collection)
    return collection


@pytest.fixture
def private_collection(db_session: Session, test_user: User) -> Collection:
    collection = Collection(
        name="Private Collection",
        description="A private collection",
        user_id=test_user.id,
        is_public=False,
    )
    db_session.add(collection)
    db_session.commit()
    db_session.refresh(collection)
    return collection


@pytest.fixture
def collection_with_tracks(
    db_session: Session, test_user: User, public_track: Track, private_track: Track
) -> Collection:
    collection = Collection(
        name="Collection With Tracks",
        description="A collection with multiple tracks",
        user_id=test_user.id,
        is_public=True,
    )
    db_session.add(collection)
    db_session.commit()
    db_session.refresh(collection)

    ct1 = CollectionTrack(
        collection_id=collection.id,
        track_id=public_track.id,
        position=0,
    )
    ct2 = CollectionTrack(
        collection_id=collection.id,
        track_id=private_track.id,
        position=1,
    )
    db_session.add_all([ct1, ct2])
    db_session.commit()
    db_session.refresh(collection)
    return collection
