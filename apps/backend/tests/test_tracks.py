import io
import os
import tempfile

from fastapi.testclient import TestClient
from sqlalchemy.orm import Session

from backend.models.track import AttachmentType, Track, TrackAttachment
from backend.models.user import User


class TestListTracks:
    def test_list_tracks_anonymous_sees_only_public(
        self, client: TestClient, public_track: Track, private_track: Track
    ):
        response = client.get("/tracks")
        assert response.status_code == 200
        data = response.json()
        assert data["total"] == 1
        assert len(data["items"]) == 1
        assert data["items"][0]["id"] == public_track.id

    def test_list_tracks_owner_sees_own_tracks(
        self,
        client: TestClient,
        public_track: Track,
        private_track: Track,
        auth_headers: dict,
    ):
        response = client.get("/tracks", headers=auth_headers)
        assert response.status_code == 200
        data = response.json()
        assert data["total"] == 2
        track_ids = {item["id"] for item in data["items"]}
        assert public_track.id in track_ids
        assert private_track.id in track_ids

    def test_list_tracks_other_user_sees_only_public(
        self,
        client: TestClient,
        public_track: Track,
        private_track: Track,
        other_auth_headers: dict,
    ):
        response = client.get("/tracks", headers=other_auth_headers)
        assert response.status_code == 200
        data = response.json()
        assert data["total"] == 1
        assert data["items"][0]["id"] == public_track.id

    def test_list_tracks_admin_sees_all(
        self,
        client: TestClient,
        public_track: Track,
        private_track: Track,
        admin_auth_headers: dict,
    ):
        response = client.get("/tracks", headers=admin_auth_headers)
        assert response.status_code == 200
        data = response.json()
        assert data["total"] == 2

    def test_list_tracks_filter_by_tag(
        self, client: TestClient, public_track: Track, auth_headers: dict
    ):
        response = client.get("/tracks?tag=rock", headers=auth_headers)
        assert response.status_code == 200
        data = response.json()
        assert data["total"] == 1
        assert data["items"][0]["id"] == public_track.id

    def test_list_tracks_filter_by_user_id(
        self,
        client: TestClient,
        public_track: Track,
        test_user: User,
        auth_headers: dict,
    ):
        response = client.get(f"/tracks?user_id={test_user.id}", headers=auth_headers)
        assert response.status_code == 200
        data = response.json()
        assert data["total"] >= 1

    def test_list_tracks_pagination(
        self, client: TestClient, public_track: Track, auth_headers: dict
    ):
        response = client.get("/tracks?page=1&page_size=1", headers=auth_headers)
        assert response.status_code == 200
        data = response.json()
        assert data["page"] == 1
        assert data["page_size"] == 1
        assert len(data["items"]) <= 1


class TestCreateTrack:
    def test_create_track_success(self, client: TestClient, auth_headers: dict):
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
                        "title": "New Track",
                        "description": "A test track",
                        "is_public": "true",
                        "tags": "test,new",
                    },
                )
        finally:
            os.unlink(temp_path)

        assert response.status_code == 201
        data = response.json()
        assert data["title"] == "New Track"
        assert data["description"] == "A test track"
        assert data["is_public"] is True
        assert data["tags"] == "test,new"
        assert data["original_filename"] == "test.mp3"
        assert data["mime_type"] == "audio/mpeg"
        assert data["slug"] == "new-track"
        assert data["processing_status"] == "processing"

    def test_create_track_requires_auth(self, client: TestClient):
        response = client.post(
            "/tracks",
            files={"file": ("test.mp3", io.BytesIO(b"content"), "audio/mpeg")},
            data={"title": "New Track"},
        )
        assert response.status_code == 401

    def test_create_track_invalid_file_type(
        self, client: TestClient, auth_headers: dict
    ):
        response = client.post(
            "/tracks",
            headers=auth_headers,
            files={"file": ("test.txt", io.BytesIO(b"content"), "text/plain")},
            data={"title": "New Track"},
        )
        assert response.status_code == 400
        assert "Invalid file type" in response.json()["detail"]


class TestGetTrack:
    def test_get_public_track_anonymous(self, client: TestClient, public_track: Track):
        response = client.get(f"/tracks/{public_track.id}")
        assert response.status_code == 200
        data = response.json()
        assert data["id"] == public_track.id
        assert data["title"] == public_track.title

    def test_get_private_track_anonymous_returns_404(
        self, client: TestClient, private_track: Track
    ):
        response = client.get(f"/tracks/{private_track.id}")
        assert response.status_code == 404

    def test_get_private_track_owner(
        self, client: TestClient, private_track: Track, auth_headers: dict
    ):
        response = client.get(f"/tracks/{private_track.id}", headers=auth_headers)
        assert response.status_code == 200
        data = response.json()
        assert data["id"] == private_track.id

    def test_get_private_track_other_user_returns_404(
        self, client: TestClient, private_track: Track, other_auth_headers: dict
    ):
        response = client.get(f"/tracks/{private_track.id}", headers=other_auth_headers)
        assert response.status_code == 404

    def test_get_private_track_admin(
        self, client: TestClient, private_track: Track, admin_auth_headers: dict
    ):
        response = client.get(f"/tracks/{private_track.id}", headers=admin_auth_headers)
        assert response.status_code == 200
        data = response.json()
        assert data["id"] == private_track.id

    def test_get_track_with_attachments(
        self, client: TestClient, track_with_attachments: Track
    ):
        response = client.get(f"/tracks/{track_with_attachments.id}")
        assert response.status_code == 200
        data = response.json()
        assert len(data["attachments"]) == 2

    def test_get_nonexistent_track(self, client: TestClient):
        response = client.get("/tracks/99999")
        assert response.status_code == 404


class TestUpdateTrack:
    def test_update_track_owner(
        self, client: TestClient, public_track: Track, auth_headers: dict
    ):
        response = client.patch(
            f"/tracks/{public_track.id}",
            headers=auth_headers,
            json={"title": "Updated Title", "tags": "updated,tags"},
        )
        assert response.status_code == 200
        data = response.json()
        assert data["title"] == "Updated Title"
        assert data["tags"] == "updated,tags"

    def test_update_track_other_user_forbidden(
        self, client: TestClient, public_track: Track, other_auth_headers: dict
    ):
        response = client.patch(
            f"/tracks/{public_track.id}",
            headers=other_auth_headers,
            json={"title": "Hacked Title"},
        )
        assert response.status_code == 403

    def test_update_track_admin(
        self, client: TestClient, public_track: Track, admin_auth_headers: dict
    ):
        response = client.patch(
            f"/tracks/{public_track.id}",
            headers=admin_auth_headers,
            json={"title": "Admin Updated"},
        )
        assert response.status_code == 200
        data = response.json()
        assert data["title"] == "Admin Updated"

    def test_update_track_requires_auth(self, client: TestClient, public_track: Track):
        response = client.patch(
            f"/tracks/{public_track.id}", json={"title": "New Title"}
        )
        assert response.status_code == 401

    def test_update_nonexistent_track(self, client: TestClient, auth_headers: dict):
        response = client.patch(
            "/tracks/99999", headers=auth_headers, json={"title": "New Title"}
        )
        assert response.status_code == 404


class TestDeleteTrack:
    def test_delete_track_owner(
        self,
        client: TestClient,
        db_session: Session,
        public_track: Track,
        auth_headers: dict,
    ):
        track_id = public_track.id
        response = client.delete(f"/tracks/{track_id}", headers=auth_headers)
        assert response.status_code == 204
        assert db_session.query(Track).filter(Track.id == track_id).first() is None

    def test_delete_track_other_user_forbidden(
        self, client: TestClient, public_track: Track, other_auth_headers: dict
    ):
        response = client.delete(
            f"/tracks/{public_track.id}", headers=other_auth_headers
        )
        assert response.status_code == 403

    def test_delete_track_admin(
        self,
        client: TestClient,
        db_session: Session,
        public_track: Track,
        admin_auth_headers: dict,
    ):
        track_id = public_track.id
        response = client.delete(f"/tracks/{track_id}", headers=admin_auth_headers)
        assert response.status_code == 204
        assert db_session.query(Track).filter(Track.id == track_id).first() is None

    def test_delete_track_requires_auth(self, client: TestClient, public_track: Track):
        response = client.delete(f"/tracks/{public_track.id}")
        assert response.status_code == 401

    def test_delete_nonexistent_track(self, client: TestClient, auth_headers: dict):
        response = client.delete("/tracks/99999", headers=auth_headers)
        assert response.status_code == 404


class TestListAttachments:
    def test_list_attachments(self, client: TestClient, track_with_attachments: Track):
        response = client.get(f"/tracks/{track_with_attachments.id}/attachments")
        assert response.status_code == 200
        data = response.json()
        assert len(data) == 2

    def test_list_attachments_filter_by_type(
        self, client: TestClient, track_with_attachments: Track
    ):
        response = client.get(
            f"/tracks/{track_with_attachments.id}/attachments?type=note"
        )
        assert response.status_code == 200
        data = response.json()
        assert len(data) == 1
        assert data[0]["type"] == "note"

    def test_list_attachments_private_track_anonymous(
        self, client: TestClient, private_track: Track
    ):
        response = client.get(f"/tracks/{private_track.id}/attachments")
        assert response.status_code == 404

    def test_list_attachments_nonexistent_track(self, client: TestClient):
        response = client.get("/tracks/99999/attachments")
        assert response.status_code == 404


class TestCreateAttachment:
    def test_create_note_attachment(
        self, client: TestClient, public_track: Track, auth_headers: dict
    ):
        response = client.post(
            f"/tracks/{public_track.id}/attachments",
            headers=auth_headers,
            data={
                "type": "note",
                "content": "This is a test note",
                "caption": "My note",
            },
        )
        assert response.status_code == 201
        data = response.json()
        assert data["type"] == "note"
        assert data["content"] == "This is a test note"
        assert data["caption"] == "My note"

    def test_create_note_without_content_fails(
        self, client: TestClient, public_track: Track, auth_headers: dict
    ):
        response = client.post(
            f"/tracks/{public_track.id}/attachments",
            headers=auth_headers,
            data={"type": "note"},
        )
        assert response.status_code == 400
        assert "Content is required" in response.json()["detail"]

    def test_create_image_attachment(
        self, client: TestClient, public_track: Track, auth_headers: dict
    ):
        response = client.post(
            f"/tracks/{public_track.id}/attachments",
            headers=auth_headers,
            files={"file": ("test.jpg", io.BytesIO(b"fake image"), "image/jpeg")},
            data={"type": "image", "caption": "Album art"},
        )
        assert response.status_code == 201
        data = response.json()
        assert data["type"] == "image"
        assert data["original_filename"] == "test.jpg"
        assert data["caption"] == "Album art"

    def test_create_image_without_file_fails(
        self, client: TestClient, public_track: Track, auth_headers: dict
    ):
        response = client.post(
            f"/tracks/{public_track.id}/attachments",
            headers=auth_headers,
            data={"type": "image"},
        )
        assert response.status_code == 400
        assert "File is required" in response.json()["detail"]

    def test_create_image_invalid_type(
        self, client: TestClient, public_track: Track, auth_headers: dict
    ):
        response = client.post(
            f"/tracks/{public_track.id}/attachments",
            headers=auth_headers,
            files={"file": ("test.txt", io.BytesIO(b"text"), "text/plain")},
            data={"type": "image"},
        )
        assert response.status_code == 400
        assert "Invalid file type" in response.json()["detail"]

    def test_create_video_attachment(
        self, client: TestClient, public_track: Track, auth_headers: dict
    ):
        response = client.post(
            f"/tracks/{public_track.id}/attachments",
            headers=auth_headers,
            files={"file": ("test.mp4", io.BytesIO(b"fake video"), "video/mp4")},
            data={"type": "video", "caption": "Music video"},
        )
        assert response.status_code == 201
        data = response.json()
        assert data["type"] == "video"

    def test_create_attachment_requires_auth(
        self, client: TestClient, public_track: Track
    ):
        response = client.post(
            f"/tracks/{public_track.id}/attachments",
            data={"type": "note", "content": "test"},
        )
        assert response.status_code == 401

    def test_create_attachment_other_user_forbidden(
        self, client: TestClient, public_track: Track, other_auth_headers: dict
    ):
        response = client.post(
            f"/tracks/{public_track.id}/attachments",
            headers=other_auth_headers,
            data={"type": "note", "content": "test"},
        )
        assert response.status_code == 403

    def test_create_attachment_admin(
        self, client: TestClient, public_track: Track, admin_auth_headers: dict
    ):
        response = client.post(
            f"/tracks/{public_track.id}/attachments",
            headers=admin_auth_headers,
            data={"type": "note", "content": "Admin note"},
        )
        assert response.status_code == 201


class TestUpdateAttachment:
    def test_update_note_attachment(
        self,
        client: TestClient,
        db_session: Session,
        track_with_attachments: Track,
        auth_headers: dict,
    ):
        note = (
            db_session.query(TrackAttachment)
            .filter(
                TrackAttachment.track_id == track_with_attachments.id,
                TrackAttachment.type == AttachmentType.NOTE,
            )
            .first()
        )
        response = client.patch(
            f"/tracks/{track_with_attachments.id}/attachments/{note.id}",
            headers=auth_headers,
            json={"content": "Updated note content"},
        )
        assert response.status_code == 200
        data = response.json()
        assert data["content"] == "Updated note content"

    def test_update_image_attachment_fails(
        self,
        client: TestClient,
        db_session: Session,
        track_with_attachments: Track,
        auth_headers: dict,
    ):
        image = (
            db_session.query(TrackAttachment)
            .filter(
                TrackAttachment.track_id == track_with_attachments.id,
                TrackAttachment.type == AttachmentType.IMAGE,
            )
            .first()
        )
        response = client.patch(
            f"/tracks/{track_with_attachments.id}/attachments/{image.id}",
            headers=auth_headers,
            json={"caption": "New caption"},
        )
        assert response.status_code == 400
        assert "Only note attachments" in response.json()["detail"]

    def test_update_attachment_requires_auth(
        self, client: TestClient, db_session: Session, track_with_attachments: Track
    ):
        note = (
            db_session.query(TrackAttachment)
            .filter(TrackAttachment.track_id == track_with_attachments.id)
            .first()
        )
        response = client.patch(
            f"/tracks/{track_with_attachments.id}/attachments/{note.id}",
            json={"content": "test"},
        )
        assert response.status_code == 401

    def test_update_attachment_other_user_forbidden(
        self,
        client: TestClient,
        db_session: Session,
        track_with_attachments: Track,
        other_auth_headers: dict,
    ):
        note = (
            db_session.query(TrackAttachment)
            .filter(
                TrackAttachment.track_id == track_with_attachments.id,
                TrackAttachment.type == AttachmentType.NOTE,
            )
            .first()
        )
        response = client.patch(
            f"/tracks/{track_with_attachments.id}/attachments/{note.id}",
            headers=other_auth_headers,
            json={"content": "Hacked"},
        )
        assert response.status_code == 403

    def test_update_nonexistent_attachment(
        self, client: TestClient, public_track: Track, auth_headers: dict
    ):
        response = client.patch(
            f"/tracks/{public_track.id}/attachments/99999",
            headers=auth_headers,
            json={"content": "test"},
        )
        assert response.status_code == 404


class TestDeleteAttachment:
    def test_delete_attachment_owner(
        self,
        client: TestClient,
        db_session: Session,
        track_with_attachments: Track,
        auth_headers: dict,
    ):
        note = (
            db_session.query(TrackAttachment)
            .filter(
                TrackAttachment.track_id == track_with_attachments.id,
                TrackAttachment.type == AttachmentType.NOTE,
            )
            .first()
        )
        note_id = note.id
        response = client.delete(
            f"/tracks/{track_with_attachments.id}/attachments/{note_id}",
            headers=auth_headers,
        )
        assert response.status_code == 204
        assert (
            db_session.query(TrackAttachment)
            .filter(TrackAttachment.id == note_id)
            .first()
            is None
        )

    def test_delete_attachment_other_user_forbidden(
        self,
        client: TestClient,
        db_session: Session,
        track_with_attachments: Track,
        other_auth_headers: dict,
    ):
        note = (
            db_session.query(TrackAttachment)
            .filter(TrackAttachment.track_id == track_with_attachments.id)
            .first()
        )
        response = client.delete(
            f"/tracks/{track_with_attachments.id}/attachments/{note.id}",
            headers=other_auth_headers,
        )
        assert response.status_code == 403

    def test_delete_attachment_admin(
        self,
        client: TestClient,
        db_session: Session,
        track_with_attachments: Track,
        admin_auth_headers: dict,
    ):
        note = (
            db_session.query(TrackAttachment)
            .filter(
                TrackAttachment.track_id == track_with_attachments.id,
                TrackAttachment.type == AttachmentType.NOTE,
            )
            .first()
        )
        note_id = note.id
        response = client.delete(
            f"/tracks/{track_with_attachments.id}/attachments/{note_id}",
            headers=admin_auth_headers,
        )
        assert response.status_code == 204

    def test_delete_attachment_requires_auth(
        self, client: TestClient, db_session: Session, track_with_attachments: Track
    ):
        note = (
            db_session.query(TrackAttachment)
            .filter(TrackAttachment.track_id == track_with_attachments.id)
            .first()
        )
        response = client.delete(
            f"/tracks/{track_with_attachments.id}/attachments/{note.id}"
        )
        assert response.status_code == 401

    def test_delete_nonexistent_attachment(
        self, client: TestClient, public_track: Track, auth_headers: dict
    ):
        response = client.delete(
            f"/tracks/{public_track.id}/attachments/99999", headers=auth_headers
        )
        assert response.status_code == 404
