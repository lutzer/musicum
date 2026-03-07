import io
import json
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

    def test_update_track_title_updates_slug(
        self, client: TestClient, public_track: Track, auth_headers: dict
    ):
        response = client.patch(
            f"/tracks/{public_track.id}",
            headers=auth_headers,
            json={"title": "Brand New Title"},
        )
        assert response.status_code == 200
        data = response.json()
        assert data["title"] == "Brand New Title"
        assert data["slug"] == "brand-new-title"

    def test_update_track_title_keeps_unique_slug(
        self,
        client: TestClient,
        db_session: Session,
        public_track: Track,
        auth_headers: dict,
    ):
        # Create another track with a specific slug
        other_track = Track(
            slug="target-slug",
            title="Target Slug",
            source_path="/fake/path.mp3",
            original_filename="test.mp3",
            file_size=1024,
            mime_type="audio/mpeg",
            user_id=public_track.user_id,
            is_public=True,
            processing_status="ready",
        )
        db_session.add(other_track)
        db_session.commit()

        # Try to update public_track to have the same title
        response = client.patch(
            f"/tracks/{public_track.id}",
            headers=auth_headers,
            json={"title": "Target Slug"},
        )
        assert response.status_code == 200
        data = response.json()
        assert data["title"] == "Target Slug"
        # Slug should be unique, not "target-slug"
        assert data["slug"] != "target-slug"
        assert data["slug"].startswith("target-slug-")

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

    def test_update_image_caption(
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
        assert response.status_code == 200
        data = response.json()
        assert data["caption"] == "New caption"

    def test_update_image_content_fails(
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
            json={"content": "Some content"},
        )
        assert response.status_code == 400
        assert "Content can only be updated" in response.json()["detail"]

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


class TestReorderAttachments:
    def test_reorder_attachments(
        self,
        client: TestClient,
        db_session: Session,
        track_with_attachments: Track,
        auth_headers: dict,
    ):
        attachments = (
            db_session.query(TrackAttachment)
            .filter(TrackAttachment.track_id == track_with_attachments.id)
            .order_by(TrackAttachment.position)
            .all()
        )
        original_order = [a.id for a in attachments]
        reversed_order = list(reversed(original_order))

        response = client.put(
            f"/tracks/{track_with_attachments.id}/attachments/reorder",
            headers=auth_headers,
            json={"attachment_ids": reversed_order},
        )
        assert response.status_code == 200
        data = response.json()
        assert [a["id"] for a in data] == reversed_order

    def test_reorder_attachments_requires_auth(
        self, client: TestClient, track_with_attachments: Track
    ):
        response = client.put(
            f"/tracks/{track_with_attachments.id}/attachments/reorder",
            json={"attachment_ids": [1, 2]},
        )
        assert response.status_code == 401

    def test_reorder_attachments_other_user_forbidden(
        self,
        client: TestClient,
        track_with_attachments: Track,
        other_auth_headers: dict,
    ):
        response = client.put(
            f"/tracks/{track_with_attachments.id}/attachments/reorder",
            headers=other_auth_headers,
            json={"attachment_ids": [1, 2]},
        )
        assert response.status_code == 403

    def test_reorder_attachments_nonexistent_track(
        self, client: TestClient, auth_headers: dict
    ):
        response = client.put(
            "/tracks/99999/attachments/reorder",
            headers=auth_headers,
            json={"attachment_ids": [1, 2]},
        )
        assert response.status_code == 404


class TestGetAttachmentFile:
    def test_get_note_file_fails(
        self,
        client: TestClient,
        db_session: Session,
        track_with_attachments: Track,
    ):
        note = (
            db_session.query(TrackAttachment)
            .filter(
                TrackAttachment.track_id == track_with_attachments.id,
                TrackAttachment.type == AttachmentType.NOTE,
            )
            .first()
        )
        response = client.get(
            f"/tracks/{track_with_attachments.id}/attachments/{note.id}/file"
        )
        assert response.status_code == 400
        assert "Notes do not have files" in response.json()["detail"]

    def test_get_attachment_file_nonexistent_track(self, client: TestClient):
        response = client.get("/tracks/99999/attachments/1/file")
        assert response.status_code == 404

    def test_get_attachment_file_nonexistent_attachment(
        self, client: TestClient, public_track: Track
    ):
        response = client.get(f"/tracks/{public_track.id}/attachments/99999/file")
        assert response.status_code == 404

    def test_get_attachment_file_private_track_anonymous(
        self,
        client: TestClient,
        db_session: Session,
        private_track: Track,
    ):
        attachment = TrackAttachment(
            track_id=private_track.id,
            type=AttachmentType.IMAGE,
            path="/fake/path.jpg",
            original_filename="test.jpg",
            position=0,
            processing_status="ready",
        )
        db_session.add(attachment)
        db_session.commit()
        db_session.refresh(attachment)

        response = client.get(
            f"/tracks/{private_track.id}/attachments/{attachment.id}/file"
        )
        assert response.status_code == 404


class TestAttachmentPosition:
    def test_attachment_has_position_in_response(
        self, client: TestClient, track_with_attachments: Track
    ):
        response = client.get(f"/tracks/{track_with_attachments.id}/attachments")
        assert response.status_code == 200
        data = response.json()
        for attachment in data:
            assert "position" in attachment
            assert isinstance(attachment["position"], int)

    def test_attachment_has_file_url_for_image(
        self, client: TestClient, track_with_attachments: Track
    ):
        response = client.get(f"/tracks/{track_with_attachments.id}/attachments")
        assert response.status_code == 200
        data = response.json()
        image = next((a for a in data if a["type"] == "image"), None)
        assert image is not None
        assert image["file_url"] is not None
        assert (
            f"/tracks/{track_with_attachments.id}/attachments/{image['id']}/file"
            in image["file_url"]
        )

    def test_attachment_has_no_file_url_for_note(
        self, client: TestClient, track_with_attachments: Track
    ):
        response = client.get(f"/tracks/{track_with_attachments.id}/attachments")
        assert response.status_code == 200
        data = response.json()
        note = next((a for a in data if a["type"] == "note"), None)
        assert note is not None
        assert note["file_url"] is None


class TestAttachmentFileDeletion:
    def test_delete_attachment_removes_original_file(
        self,
        client: TestClient,
        db_session: Session,
        public_track: Track,
        auth_headers: dict,
    ):
        with tempfile.TemporaryDirectory() as temp_dir:
            # Create attachment file
            attachment_path = os.path.join(temp_dir, "test_image.jpg")
            with open(attachment_path, "wb") as f:
                f.write(b"fake image data")

            attachment = TrackAttachment(
                track_id=public_track.id,
                type=AttachmentType.IMAGE,
                path=attachment_path,
                original_filename="test_image.jpg",
                position=0,
                processing_status="ready",
            )
            db_session.add(attachment)
            db_session.commit()
            db_session.refresh(attachment)

            assert os.path.exists(attachment_path)

            response = client.delete(
                f"/tracks/{public_track.id}/attachments/{attachment.id}",
                headers=auth_headers,
            )
            assert response.status_code == 204
            assert not os.path.exists(attachment_path)

    def test_delete_attachment_removes_processed_file(
        self,
        client: TestClient,
        db_session: Session,
        public_track: Track,
        auth_headers: dict,
    ):
        with tempfile.TemporaryDirectory() as temp_dir:
            # Create attachment files
            original_path = os.path.join(temp_dir, "test_image.jpg")
            processed_path = os.path.join(temp_dir, "test_image_processed.jpg")
            with open(original_path, "wb") as f:
                f.write(b"fake original image")
            with open(processed_path, "wb") as f:
                f.write(b"fake processed image")

            attachment = TrackAttachment(
                track_id=public_track.id,
                type=AttachmentType.IMAGE,
                path=original_path,
                processed_path=processed_path,
                original_filename="test_image.jpg",
                position=0,
                processing_status="ready",
            )
            db_session.add(attachment)
            db_session.commit()
            db_session.refresh(attachment)

            assert os.path.exists(original_path)
            assert os.path.exists(processed_path)

            response = client.delete(
                f"/tracks/{public_track.id}/attachments/{attachment.id}",
                headers=auth_headers,
            )
            assert response.status_code == 204
            assert not os.path.exists(original_path)
            assert not os.path.exists(processed_path)


class TestTrackFileDeletion:
    def test_delete_track_removes_track_folder(
        self,
        client: TestClient,
        db_session: Session,
        test_user: User,
        auth_headers: dict,
    ):
        with tempfile.TemporaryDirectory() as temp_dir:
            # Create track folder with source file
            track_dir = os.path.join(temp_dir, "test-track")
            os.makedirs(track_dir)
            source_path = os.path.join(track_dir, "audio.mp3")
            with open(source_path, "wb") as f:
                f.write(b"fake audio data")

            track = Track(
                slug="test-track-delete",
                title="Test Track Delete",
                source_path=source_path,
                original_filename="audio.mp3",
                file_size=1024,
                mime_type="audio/mpeg",
                user_id=test_user.id,
                is_public=True,
                processing_status="ready",
            )
            db_session.add(track)
            db_session.commit()
            db_session.refresh(track)

            assert os.path.exists(track_dir)

            response = client.delete(f"/tracks/{track.id}", headers=auth_headers)
            assert response.status_code == 204
            assert not os.path.exists(track_dir)

    def test_delete_track_removes_converted_file(
        self,
        client: TestClient,
        db_session: Session,
        test_user: User,
        auth_headers: dict,
    ):
        with tempfile.TemporaryDirectory() as temp_dir:
            # Create track folder with source file
            track_dir = os.path.join(temp_dir, "test-track")
            os.makedirs(track_dir)
            source_path = os.path.join(track_dir, "audio.mp3")
            converted_path = os.path.join(temp_dir, "converted.mp3")
            with open(source_path, "wb") as f:
                f.write(b"fake audio data")
            with open(converted_path, "wb") as f:
                f.write(b"fake converted audio data")

            track = Track(
                slug="test-track-delete-converted",
                title="Test Track Delete Converted",
                source_path=source_path,
                converted_path=converted_path,
                original_filename="audio.mp3",
                file_size=1024,
                mime_type="audio/mpeg",
                user_id=test_user.id,
                is_public=True,
                processing_status="ready",
            )
            db_session.add(track)
            db_session.commit()
            db_session.refresh(track)

            assert os.path.exists(converted_path)

            response = client.delete(f"/tracks/{track.id}", headers=auth_headers)
            assert response.status_code == 204
            assert not os.path.exists(converted_path)

    def test_delete_track_removes_track_json(
        self,
        client: TestClient,
        db_session: Session,
        test_user: User,
        auth_headers: dict,
    ):
        with tempfile.TemporaryDirectory() as temp_dir:
            # Create track folder with source file and track.json
            track_dir = os.path.join(temp_dir, "test-track")
            os.makedirs(track_dir)
            source_path = os.path.join(track_dir, "audio.mp3")
            track_json_path = os.path.join(track_dir, "track.json")
            with open(source_path, "wb") as f:
                f.write(b"fake audio data")
            with open(track_json_path, "w") as f:
                json.dump({"title": "Test"}, f)

            track = Track(
                slug="test-track-delete-json",
                title="Test Track Delete JSON",
                source_path=source_path,
                original_filename="audio.mp3",
                file_size=1024,
                mime_type="audio/mpeg",
                user_id=test_user.id,
                is_public=True,
                processing_status="ready",
            )
            db_session.add(track)
            db_session.commit()
            db_session.refresh(track)

            assert os.path.exists(track_json_path)

            response = client.delete(f"/tracks/{track.id}", headers=auth_headers)
            assert response.status_code == 204
            assert not os.path.exists(track_json_path)
            assert not os.path.exists(track_dir)


class TestTrackJsonSync:
    def test_track_json_updated_on_track_edit(
        self,
        client: TestClient,
        db_session: Session,
        test_user: User,
        auth_headers: dict,
    ):
        with tempfile.TemporaryDirectory() as temp_dir:
            # Create track folder with source file
            track_dir = os.path.join(temp_dir, "test-track")
            os.makedirs(track_dir)
            source_path = os.path.join(track_dir, "audio.mp3")
            with open(source_path, "wb") as f:
                f.write(b"fake audio data")

            track = Track(
                slug="test-track-edit",
                title="Original Title",
                description="Original description",
                source_path=source_path,
                original_filename="audio.mp3",
                file_size=1024,
                mime_type="audio/mpeg",
                user_id=test_user.id,
                is_public=True,
                processing_status="ready",
            )
            db_session.add(track)
            db_session.commit()
            db_session.refresh(track)

            response = client.patch(
                f"/tracks/{track.id}",
                headers=auth_headers,
                json={"title": "Updated Title", "description": "Updated description"},
            )
            assert response.status_code == 200

            track_json_path = os.path.join(track_dir, "track.json")
            assert os.path.exists(track_json_path)

            with open(track_json_path) as f:
                metadata = json.load(f)

            assert metadata["data"]["title"] == "Updated Title"
            assert metadata["data"]["description"] == "Updated description"

    def test_track_json_updated_on_attachment_add(
        self,
        client: TestClient,
        db_session: Session,
        test_user: User,
        auth_headers: dict,
    ):
        with tempfile.TemporaryDirectory() as temp_dir:
            # Create track folder with source file
            track_dir = os.path.join(temp_dir, "test-track")
            os.makedirs(track_dir)
            source_path = os.path.join(track_dir, "audio.mp3")
            with open(source_path, "wb") as f:
                f.write(b"fake audio data")

            track = Track(
                slug="test-track-attach",
                title="Track with Attachment",
                source_path=source_path,
                original_filename="audio.mp3",
                file_size=1024,
                mime_type="audio/mpeg",
                user_id=test_user.id,
                is_public=True,
                processing_status="ready",
            )
            db_session.add(track)
            db_session.commit()
            db_session.refresh(track)

            response = client.post(
                f"/tracks/{track.id}/attachments",
                headers=auth_headers,
                data={
                    "type": "note",
                    "content": "Test note",
                    "caption": "Note caption",
                },
            )
            assert response.status_code == 201
            attachment_data = response.json()

            track_json_path = os.path.join(track_dir, "track.json")
            assert os.path.exists(track_json_path)

            with open(track_json_path) as f:
                metadata = json.load(f)

            assert "attachments" in metadata["data"]
            assert len(metadata["data"]["attachments"]) == 1
            assert metadata["data"]["attachments"][0]["id"] == attachment_data["id"]
            assert metadata["data"]["attachments"][0]["type"] == "note"
            assert metadata["data"]["attachments"][0]["caption"] == "Note caption"

    def test_track_json_updated_on_attachment_delete(
        self,
        client: TestClient,
        db_session: Session,
        test_user: User,
        auth_headers: dict,
    ):
        with tempfile.TemporaryDirectory() as temp_dir:
            # Create track folder with source file
            track_dir = os.path.join(temp_dir, "test-track")
            os.makedirs(track_dir)
            source_path = os.path.join(track_dir, "audio.mp3")
            with open(source_path, "wb") as f:
                f.write(b"fake audio data")

            track = Track(
                slug="test-track-attach-del",
                title="Track with Attachment Delete",
                source_path=source_path,
                original_filename="audio.mp3",
                file_size=1024,
                mime_type="audio/mpeg",
                user_id=test_user.id,
                is_public=True,
                processing_status="ready",
            )
            db_session.add(track)
            db_session.commit()
            db_session.refresh(track)

            # Add two attachments
            attachment1 = TrackAttachment(
                track_id=track.id,
                type=AttachmentType.NOTE,
                content="Note 1",
                position=0,
                processing_status="ready",
            )
            attachment2 = TrackAttachment(
                track_id=track.id,
                type=AttachmentType.NOTE,
                content="Note 2",
                position=1,
                processing_status="ready",
            )
            db_session.add_all([attachment1, attachment2])
            db_session.commit()
            db_session.refresh(attachment1)
            db_session.refresh(attachment2)

            # Delete first attachment
            response = client.delete(
                f"/tracks/{track.id}/attachments/{attachment1.id}",
                headers=auth_headers,
            )
            assert response.status_code == 204

            track_json_path = os.path.join(track_dir, "track.json")
            assert os.path.exists(track_json_path)

            with open(track_json_path) as f:
                metadata = json.load(f)

            assert "attachments" in metadata["data"]
            assert len(metadata["data"]["attachments"]) == 1
            assert metadata["data"]["attachments"][0]["id"] == attachment2.id

    def test_track_json_contains_attachments(
        self,
        client: TestClient,
        db_session: Session,
        test_user: User,
        auth_headers: dict,
    ):
        with tempfile.TemporaryDirectory() as temp_dir:
            # Create track folder with source file
            track_dir = os.path.join(temp_dir, "test-track")
            os.makedirs(track_dir)
            source_path = os.path.join(track_dir, "audio.mp3")
            with open(source_path, "wb") as f:
                f.write(b"fake audio data")

            track = Track(
                slug="test-track-attachments",
                title="Track with Multiple Attachments",
                source_path=source_path,
                original_filename="audio.mp3",
                file_size=1024,
                mime_type="audio/mpeg",
                user_id=test_user.id,
                is_public=True,
                processing_status="ready",
            )
            db_session.add(track)
            db_session.commit()
            db_session.refresh(track)

            # Add multiple attachments
            note = TrackAttachment(
                track_id=track.id,
                type=AttachmentType.NOTE,
                content="Test note",
                caption="Note caption",
                position=0,
                processing_status="ready",
            )
            image = TrackAttachment(
                track_id=track.id,
                type=AttachmentType.IMAGE,
                path="/fake/path/image.jpg",
                original_filename="image.jpg",
                caption="Image caption",
                position=1,
                processing_status="ready",
            )
            db_session.add_all([note, image])
            db_session.commit()
            db_session.refresh(note)
            db_session.refresh(image)

            # Trigger track.json update by editing track
            response = client.patch(
                f"/tracks/{track.id}",
                headers=auth_headers,
                json={"description": "Updated"},
            )
            assert response.status_code == 200

            track_json_path = os.path.join(track_dir, "track.json")
            with open(track_json_path) as f:
                metadata = json.load(f)

            assert "attachments" in metadata["data"]
            assert len(metadata["data"]["attachments"]) == 2

            note_meta = next(
                (a for a in metadata["data"]["attachments"] if a["type"] == "note"), None
            )
            image_meta = next(
                (a for a in metadata["data"]["attachments"] if a["type"] == "image"), None
            )

            assert note_meta is not None
            assert note_meta["id"] == note.id
            assert note_meta["caption"] == "Note caption"
            assert note_meta["position"] == 0

            assert image_meta is not None
            assert image_meta["id"] == image.id
            assert image_meta["caption"] == "Image caption"
            assert image_meta["position"] == 1
