import io
import tempfile

from fastapi.testclient import TestClient
from sqlalchemy.orm import Session

from backend.models.track import Track
from backend.models.user import User
from backend.services.track_service import generate_unique_slug


class TestSlugGeneration:
    def test_generate_slug_from_title(self, db_session: Session):
        slug = generate_unique_slug(db_session, "My Awesome Song")
        assert slug == "my-awesome-song"

    def test_generate_slug_handles_special_characters(self, db_session: Session):
        slug = generate_unique_slug(db_session, "Test & Track (remix)")
        assert "test" in slug
        assert "track" in slug

    def test_generate_slug_handles_empty_title(self, db_session: Session):
        slug = generate_unique_slug(db_session, "")
        assert slug == "track"

    def test_generate_slug_adds_suffix_for_duplicate(
        self, db_session: Session, test_user: User
    ):
        track = Track(
            slug="my-song",
            title="My Song",
            source_path="/fake/path.mp3",
            original_filename="song.mp3",
            file_size=1024,
            mime_type="audio/mpeg",
            user_id=test_user.id,
            processing_status="ready",
        )
        db_session.add(track)
        db_session.commit()

        new_slug = generate_unique_slug(db_session, "My Song")
        assert new_slug.startswith("my-song-")
        assert len(new_slug) > len("my-song")


class TestTrackCreationWithSlug:
    def test_create_track_generates_slug(self, client: TestClient, auth_headers: dict):
        with tempfile.NamedTemporaryFile(suffix=".mp3", delete=False) as f:
            f.write(b"fake audio content")
            temp_path = f.name

        try:
            with open(temp_path, "rb") as f:
                response = client.post(
                    "/tracks",
                    headers=auth_headers,
                    files={"file": ("test.mp3", f, "audio/mpeg")},
                    data={
                        "title": "My Test Track",
                        "description": "A test track",
                        "is_public": "true",
                    },
                )
        finally:
            import os

            os.unlink(temp_path)

        assert response.status_code == 201
        data = response.json()
        assert data["slug"] == "my-test-track"
        assert data["processing_status"] == "processing"

    def test_create_track_response_includes_new_fields(
        self, client: TestClient, auth_headers: dict
    ):
        response = client.post(
            "/tracks",
            headers=auth_headers,
            files={"file": ("test.mp3", io.BytesIO(b"fake audio"), "audio/mpeg")},
            data={"title": "New Track"},
        )
        assert response.status_code == 201
        data = response.json()
        assert "slug" in data
        assert "processing_status" in data
        assert "converted_path" in data


class TestTrackProcessingStatus:
    def test_track_response_includes_processing_status(
        self, client: TestClient, public_track: Track
    ):
        response = client.get(f"/tracks/{public_track.id}")
        assert response.status_code == 200
        data = response.json()
        assert data["processing_status"] == "ready"

    def test_track_list_includes_processing_status(
        self, client: TestClient, public_track: Track
    ):
        response = client.get("/tracks")
        assert response.status_code == 200
        data = response.json()
        assert len(data["items"]) > 0
        assert "processing_status" in data["items"][0]


class TestGetTrackBySlug:
    def test_get_track_by_slug(self, client: TestClient, public_track: Track):
        response = client.get(f"/tracks/by-slug/{public_track.slug}")
        assert response.status_code == 200
        data = response.json()
        assert data["id"] == public_track.id
        assert data["slug"] == public_track.slug

    def test_get_track_by_slug_not_found(self, client: TestClient):
        response = client.get("/tracks/by-slug/nonexistent-slug")
        assert response.status_code == 404

    def test_get_private_track_by_slug_anonymous_returns_404(
        self, client: TestClient, private_track: Track
    ):
        response = client.get(f"/tracks/by-slug/{private_track.slug}")
        assert response.status_code == 404

    def test_get_private_track_by_slug_owner(
        self, client: TestClient, private_track: Track, auth_headers: dict
    ):
        response = client.get(
            f"/tracks/by-slug/{private_track.slug}", headers=auth_headers
        )
        assert response.status_code == 200
        data = response.json()
        assert data["id"] == private_track.id
