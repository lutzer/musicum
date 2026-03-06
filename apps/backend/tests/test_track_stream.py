import os
import tempfile

from fastapi.testclient import TestClient
from sqlalchemy.orm import Session

from backend.models.track import Track
from backend.models.user import User


class TestStreamTrack:
    def test_stream_track_returns_original_when_not_converted(
        self, client: TestClient, db_session: Session, test_user: User
    ):
        with tempfile.NamedTemporaryFile(suffix=".wav", delete=False, mode="wb") as f:
            f.write(b"fake wav content")
            source_path = f.name

        track = Track(
            slug="stream-test-track",
            title="Stream Test Track",
            source_path=source_path,
            original_filename="test.wav",
            file_size=len(b"fake wav content"),
            mime_type="audio/wav",
            user_id=test_user.id,
            is_public=True,
            processing_status="processing",
        )
        db_session.add(track)
        db_session.commit()
        db_session.refresh(track)

        try:
            response = client.get(f"/tracks/{track.id}/stream")
            assert response.status_code == 200
            assert response.headers["content-type"] == "audio/wav"
            assert response.content == b"fake wav content"
        finally:
            os.unlink(source_path)

    def test_stream_track_returns_converted_when_ready(
        self, client: TestClient, db_session: Session, test_user: User
    ):
        with tempfile.NamedTemporaryFile(suffix=".wav", delete=False, mode="wb") as f:
            f.write(b"fake wav content")
            source_path = f.name

        with tempfile.NamedTemporaryFile(suffix=".mp3", delete=False, mode="wb") as f:
            f.write(b"fake mp3 content")
            converted_path = f.name

        track = Track(
            slug="stream-converted-track",
            title="Stream Converted Track",
            source_path=source_path,
            original_filename="test.wav",
            file_size=len(b"fake wav content"),
            mime_type="audio/wav",
            user_id=test_user.id,
            is_public=True,
            processing_status="ready",
            converted_path=converted_path,
        )
        db_session.add(track)
        db_session.commit()
        db_session.refresh(track)

        try:
            response = client.get(f"/tracks/{track.id}/stream")
            assert response.status_code == 200
            assert response.headers["content-type"] == "audio/mpeg"
            assert response.content == b"fake mp3 content"
        finally:
            os.unlink(source_path)
            os.unlink(converted_path)

    def test_stream_private_track_anonymous_returns_404(
        self, client: TestClient, private_track: Track
    ):
        response = client.get(f"/tracks/{private_track.id}/stream")
        assert response.status_code == 404

    def test_stream_private_track_owner(
        self,
        client: TestClient,
        db_session: Session,
        test_user: User,
        auth_headers: dict,
    ):
        with tempfile.NamedTemporaryFile(suffix=".mp3", delete=False, mode="wb") as f:
            f.write(b"fake audio")
            source_path = f.name

        track = Track(
            slug="private-stream-track",
            title="Private Stream Track",
            source_path=source_path,
            original_filename="test.mp3",
            file_size=len(b"fake audio"),
            mime_type="audio/mpeg",
            user_id=test_user.id,
            is_public=False,
            processing_status="ready",
        )
        db_session.add(track)
        db_session.commit()
        db_session.refresh(track)

        try:
            response = client.get(f"/tracks/{track.id}/stream", headers=auth_headers)
            assert response.status_code == 200
        finally:
            os.unlink(source_path)

    def test_stream_nonexistent_track(self, client: TestClient):
        response = client.get("/tracks/99999/stream")
        assert response.status_code == 404

    def test_stream_track_file_not_found(
        self, client: TestClient, db_session: Session, test_user: User
    ):
        track = Track(
            slug="missing-file-track",
            title="Missing File Track",
            source_path="/nonexistent/path.mp3",
            original_filename="missing.mp3",
            file_size=1024,
            mime_type="audio/mpeg",
            user_id=test_user.id,
            is_public=True,
            processing_status="ready",
        )
        db_session.add(track)
        db_session.commit()
        db_session.refresh(track)

        response = client.get(f"/tracks/{track.id}/stream")
        assert response.status_code == 404
        assert "Audio file not found" in response.json()["detail"]
