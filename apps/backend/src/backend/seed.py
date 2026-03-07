"""Seed script for development database initialization."""

from pathlib import Path

from sqlalchemy.orm import Session

from backend.config import settings
from backend.models.collection import Collection, CollectionTrack
from backend.models.track import Track
from backend.models.user import User, UserRole
from backend.services.auth_service import get_password_hash


def create_demo_user(db: Session) -> User:
    """Create a demo user for development."""
    user = User(
        username="demo",
        email="demo@example.com",
        hashed_password=get_password_hash("demo123"),
        role=UserRole.USER,
    )
    db.add(user)
    db.commit()
    db.refresh(user)
    return user


def create_sample_tracks(db: Session, user: User) -> list[Track]:
    """Create sample tracks for development."""
    tracks_data = [
        {
            "slug": "morning-ambience",
            "title": "Morning Ambience",
            "description": "Peaceful morning soundscape with birds and gentle wind",
            "tags": "ambient,nature,morning",
            "is_public": True,
        },
        {
            "slug": "urban-rhythm",
            "title": "Urban Rhythm",
            "description": "City sounds mixed into a rhythmic composition",
            "tags": "electronic,urban,beat",
            "is_public": True,
        },
        {
            "slug": "piano-sketch-01",
            "title": "Piano Sketch #01",
            "description": "Simple piano improvisation",
            "tags": "piano,acoustic,sketch",
            "is_public": True,
        },
        {
            "slug": "rainy-afternoon",
            "title": "Rainy Afternoon",
            "description": "Rain sounds with subtle melodic elements",
            "tags": "ambient,rain,relaxing",
            "is_public": False,
        },
        {
            "slug": "synth-experiment",
            "title": "Synth Experiment",
            "description": "Experimental synthesizer textures",
            "tags": "electronic,experimental,synth",
            "is_public": True,
        },
        {
            "slug": "private-track",
            "title": "Private Track",
            "description": "secret",
            "tags": "electronic,experimental,synth",
            "is_public": False,
        },
    ]

    tracks = []
    for track_data in tracks_data:
        track_dir = Path(settings.UPLOAD_DIR_TRACKS) / track_data["slug"]
        track_dir.mkdir(parents=True, exist_ok=True)

        original_filename = f"{track_data['slug']}.mp3"
        placeholder_path = track_dir / original_filename
        placeholder_path.write_bytes(b"placeholder audio content")

        track = Track(
            slug=track_data["slug"],
            title=track_data["title"],
            description=track_data["description"],
            source_path=str(placeholder_path),
            original_filename=original_filename,
            file_size=len(b"placeholder audio content"),
            mime_type="audio/mpeg",
            duration_seconds=180.0,
            user_id=user.id,
            is_public=track_data["is_public"],
            tags=track_data["tags"],
            processing_status="ready",
        )
        db.add(track)
        tracks.append(track)

    db.commit()
    for track in tracks:
        db.refresh(track)

    return tracks


def create_sample_collection(
    db: Session, user: User, tracks: list[Track], title : str, slug : str, private: bool
) -> Collection:
    """Create a sample collection with tracks."""
    collection = Collection(
        title=title,
        slug=slug,
        description="A curated selection of ambient and atmospheric recordings",
        user_id=user.id,
        is_public=private,
    )
    db.add(collection)
    db.commit()
    db.refresh(collection)

    ambient_tracks = [t for t in tracks if "ambient" in (t.tags or "")]
    for position, track in enumerate(ambient_tracks):
        ct = CollectionTrack(
            collection_id=collection.id,
            track_id=track.id,
            position=position,
        )
        db.add(ct)

    db.commit()
    db.refresh(collection)
    return collection


def seed_database(db: Session) -> None:
    """Seed the database with sample data for development."""
    existing_user = db.query(User).first()
    if existing_user:
        return

    print("Seeding database with sample data...")

    user = create_demo_user(db)
    print(f"  Created demo user: {user.email} (password: demo123)")

    tracks = create_sample_tracks(db, user)
    print(f"  Created {len(tracks)} sample tracks")

    collection = create_sample_collection(db, user, tracks, "Ambient Collection", "ambient-collection", True)
    print(f"  Created collection: {collection.title}")

    collection = create_sample_collection(db, user, tracks, "Private Collection", "private-collection", False)
    print(f"  Created collection: {collection.title}")

    print("Database seeding complete!")


def is_fresh_database(db: Session) -> bool:
    """Check if the database has no users (indicating a fresh database)."""
    return db.query(User).first() is None
