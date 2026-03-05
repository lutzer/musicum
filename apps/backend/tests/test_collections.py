from fastapi.testclient import TestClient
from sqlalchemy.orm import Session

from backend.models.collection import Collection, CollectionTrack
from backend.models.track import Track
from backend.models.user import User


class TestListCollections:
    def test_list_collections_anonymous_sees_only_public(
        self,
        client: TestClient,
        public_collection: Collection,
        private_collection: Collection,
    ):
        response = client.get("/collections")
        assert response.status_code == 200
        data = response.json()
        assert data["total"] == 1
        assert len(data["items"]) == 1
        assert data["items"][0]["id"] == public_collection.id

    def test_list_collections_owner_sees_own_collections(
        self,
        client: TestClient,
        public_collection: Collection,
        private_collection: Collection,
        auth_headers: dict,
    ):
        response = client.get("/collections", headers=auth_headers)
        assert response.status_code == 200
        data = response.json()
        assert data["total"] == 2
        collection_ids = {item["id"] for item in data["items"]}
        assert public_collection.id in collection_ids
        assert private_collection.id in collection_ids

    def test_list_collections_other_user_sees_only_public(
        self,
        client: TestClient,
        public_collection: Collection,
        private_collection: Collection,
        other_auth_headers: dict,
    ):
        response = client.get("/collections", headers=other_auth_headers)
        assert response.status_code == 200
        data = response.json()
        assert data["total"] == 1
        assert data["items"][0]["id"] == public_collection.id

    def test_list_collections_admin_sees_all(
        self,
        client: TestClient,
        public_collection: Collection,
        private_collection: Collection,
        admin_auth_headers: dict,
    ):
        response = client.get("/collections", headers=admin_auth_headers)
        assert response.status_code == 200
        data = response.json()
        assert data["total"] == 2

    def test_list_collections_filter_by_user_id(
        self,
        client: TestClient,
        public_collection: Collection,
        test_user: User,
        auth_headers: dict,
    ):
        response = client.get(
            f"/collections?user_id={test_user.id}", headers=auth_headers
        )
        assert response.status_code == 200
        data = response.json()
        assert data["total"] >= 1

    def test_list_collections_pagination(
        self, client: TestClient, public_collection: Collection, auth_headers: dict
    ):
        response = client.get("/collections?page=1&page_size=1", headers=auth_headers)
        assert response.status_code == 200
        data = response.json()
        assert data["page"] == 1
        assert data["page_size"] == 1
        assert len(data["items"]) <= 1


class TestCreateCollection:
    def test_create_collection_success(self, client: TestClient, auth_headers: dict):
        response = client.post(
            "/collections",
            headers=auth_headers,
            json={
                "name": "New Collection",
                "description": "A test collection",
                "is_public": True,
            },
        )
        assert response.status_code == 201
        data = response.json()
        assert data["name"] == "New Collection"
        assert data["description"] == "A test collection"
        assert data["is_public"] is True

    def test_create_collection_minimal(self, client: TestClient, auth_headers: dict):
        response = client.post(
            "/collections",
            headers=auth_headers,
            json={"name": "Minimal Collection"},
        )
        assert response.status_code == 201
        data = response.json()
        assert data["name"] == "Minimal Collection"
        assert data["description"] is None
        assert data["is_public"] is False

    def test_create_collection_requires_auth(self, client: TestClient):
        response = client.post(
            "/collections",
            json={"name": "New Collection"},
        )
        assert response.status_code == 401


class TestGetCollection:
    def test_get_public_collection_anonymous(
        self, client: TestClient, public_collection: Collection
    ):
        response = client.get(f"/collections/{public_collection.id}")
        assert response.status_code == 200
        data = response.json()
        assert data["id"] == public_collection.id
        assert data["name"] == public_collection.name

    def test_get_private_collection_anonymous_returns_404(
        self, client: TestClient, private_collection: Collection
    ):
        response = client.get(f"/collections/{private_collection.id}")
        assert response.status_code == 404

    def test_get_private_collection_owner(
        self, client: TestClient, private_collection: Collection, auth_headers: dict
    ):
        response = client.get(
            f"/collections/{private_collection.id}", headers=auth_headers
        )
        assert response.status_code == 200
        data = response.json()
        assert data["id"] == private_collection.id

    def test_get_private_collection_other_user_returns_404(
        self,
        client: TestClient,
        private_collection: Collection,
        other_auth_headers: dict,
    ):
        response = client.get(
            f"/collections/{private_collection.id}", headers=other_auth_headers
        )
        assert response.status_code == 404

    def test_get_private_collection_admin(
        self,
        client: TestClient,
        private_collection: Collection,
        admin_auth_headers: dict,
    ):
        response = client.get(
            f"/collections/{private_collection.id}", headers=admin_auth_headers
        )
        assert response.status_code == 200
        data = response.json()
        assert data["id"] == private_collection.id

    def test_get_collection_with_tracks(
        self, client: TestClient, collection_with_tracks: Collection
    ):
        response = client.get(f"/collections/{collection_with_tracks.id}")
        assert response.status_code == 200
        data = response.json()
        assert len(data["tracks"]) == 2
        # Verify tracks are ordered by position
        assert data["tracks"][0]["position"] == 0
        assert data["tracks"][1]["position"] == 1

    def test_get_nonexistent_collection(self, client: TestClient):
        response = client.get("/collections/99999")
        assert response.status_code == 404


class TestUpdateCollection:
    def test_update_collection_owner(
        self, client: TestClient, public_collection: Collection, auth_headers: dict
    ):
        response = client.patch(
            f"/collections/{public_collection.id}",
            headers=auth_headers,
            json={"name": "Updated Name", "description": "Updated description"},
        )
        assert response.status_code == 200
        data = response.json()
        assert data["name"] == "Updated Name"
        assert data["description"] == "Updated description"

    def test_update_collection_other_user_forbidden(
        self,
        client: TestClient,
        public_collection: Collection,
        other_auth_headers: dict,
    ):
        response = client.patch(
            f"/collections/{public_collection.id}",
            headers=other_auth_headers,
            json={"name": "Hacked Name"},
        )
        assert response.status_code == 403

    def test_update_collection_admin(
        self,
        client: TestClient,
        public_collection: Collection,
        admin_auth_headers: dict,
    ):
        response = client.patch(
            f"/collections/{public_collection.id}",
            headers=admin_auth_headers,
            json={"name": "Admin Updated"},
        )
        assert response.status_code == 200
        data = response.json()
        assert data["name"] == "Admin Updated"

    def test_update_collection_requires_auth(
        self, client: TestClient, public_collection: Collection
    ):
        response = client.patch(
            f"/collections/{public_collection.id}", json={"name": "New Name"}
        )
        assert response.status_code == 401

    def test_update_nonexistent_collection(
        self, client: TestClient, auth_headers: dict
    ):
        response = client.patch(
            "/collections/99999", headers=auth_headers, json={"name": "New Name"}
        )
        assert response.status_code == 404


class TestDeleteCollection:
    def test_delete_collection_owner(
        self,
        client: TestClient,
        db_session: Session,
        public_collection: Collection,
        auth_headers: dict,
    ):
        collection_id = public_collection.id
        response = client.delete(f"/collections/{collection_id}", headers=auth_headers)
        assert response.status_code == 204
        assert (
            db_session.query(Collection).filter(Collection.id == collection_id).first()
            is None
        )

    def test_delete_collection_other_user_forbidden(
        self,
        client: TestClient,
        public_collection: Collection,
        other_auth_headers: dict,
    ):
        response = client.delete(
            f"/collections/{public_collection.id}", headers=other_auth_headers
        )
        assert response.status_code == 403

    def test_delete_collection_admin(
        self,
        client: TestClient,
        db_session: Session,
        public_collection: Collection,
        admin_auth_headers: dict,
    ):
        collection_id = public_collection.id
        response = client.delete(
            f"/collections/{collection_id}", headers=admin_auth_headers
        )
        assert response.status_code == 204
        assert (
            db_session.query(Collection).filter(Collection.id == collection_id).first()
            is None
        )

    def test_delete_collection_requires_auth(
        self, client: TestClient, public_collection: Collection
    ):
        response = client.delete(f"/collections/{public_collection.id}")
        assert response.status_code == 401

    def test_delete_nonexistent_collection(
        self, client: TestClient, auth_headers: dict
    ):
        response = client.delete("/collections/99999", headers=auth_headers)
        assert response.status_code == 404

    def test_delete_collection_cascades_to_collection_tracks(
        self,
        client: TestClient,
        db_session: Session,
        collection_with_tracks: Collection,
        auth_headers: dict,
    ):
        collection_id = collection_with_tracks.id
        # Verify collection tracks exist
        ct_count = (
            db_session.query(CollectionTrack)
            .filter(CollectionTrack.collection_id == collection_id)
            .count()
        )
        assert ct_count == 2

        response = client.delete(f"/collections/{collection_id}", headers=auth_headers)
        assert response.status_code == 204

        # Verify collection tracks are deleted
        ct_count = (
            db_session.query(CollectionTrack)
            .filter(CollectionTrack.collection_id == collection_id)
            .count()
        )
        assert ct_count == 0


class TestListCollectionTracks:
    def test_list_collection_tracks(
        self, client: TestClient, collection_with_tracks: Collection
    ):
        response = client.get(f"/collections/{collection_with_tracks.id}/tracks")
        assert response.status_code == 200
        data = response.json()
        assert len(data) == 2
        # Verify tracks are ordered by position
        assert data[0]["position"] == 0
        assert data[1]["position"] == 1

    def test_list_collection_tracks_private_collection_anonymous(
        self, client: TestClient, private_collection: Collection
    ):
        response = client.get(f"/collections/{private_collection.id}/tracks")
        assert response.status_code == 404

    def test_list_collection_tracks_nonexistent_collection(self, client: TestClient):
        response = client.get("/collections/99999/tracks")
        assert response.status_code == 404


class TestAddTrackToCollection:
    def test_add_track_to_collection(
        self,
        client: TestClient,
        public_collection: Collection,
        public_track: Track,
        auth_headers: dict,
    ):
        response = client.post(
            f"/collections/{public_collection.id}/tracks",
            headers=auth_headers,
            json={"track_id": public_track.id},
        )
        assert response.status_code == 201
        data = response.json()
        assert data["track_id"] == public_track.id
        assert data["collection_id"] == public_collection.id

    def test_add_track_to_collection_with_position(
        self,
        client: TestClient,
        public_collection: Collection,
        public_track: Track,
        auth_headers: dict,
    ):
        response = client.post(
            f"/collections/{public_collection.id}/tracks",
            headers=auth_headers,
            json={"track_id": public_track.id, "position": 5},
        )
        assert response.status_code == 201
        data = response.json()
        assert data["position"] == 5

    def test_add_track_to_collection_requires_auth(
        self, client: TestClient, public_collection: Collection, public_track: Track
    ):
        response = client.post(
            f"/collections/{public_collection.id}/tracks",
            json={"track_id": public_track.id},
        )
        assert response.status_code == 401

    def test_add_track_to_collection_other_user_forbidden(
        self,
        client: TestClient,
        public_collection: Collection,
        public_track: Track,
        other_auth_headers: dict,
    ):
        response = client.post(
            f"/collections/{public_collection.id}/tracks",
            headers=other_auth_headers,
            json={"track_id": public_track.id},
        )
        assert response.status_code == 403

    def test_add_track_to_collection_admin(
        self,
        client: TestClient,
        public_collection: Collection,
        public_track: Track,
        admin_auth_headers: dict,
    ):
        response = client.post(
            f"/collections/{public_collection.id}/tracks",
            headers=admin_auth_headers,
            json={"track_id": public_track.id},
        )
        assert response.status_code == 201

    def test_add_nonexistent_track_to_collection(
        self, client: TestClient, public_collection: Collection, auth_headers: dict
    ):
        response = client.post(
            f"/collections/{public_collection.id}/tracks",
            headers=auth_headers,
            json={"track_id": 99999},
        )
        assert response.status_code == 404
        assert "Track not found" in response.json()["detail"]

    def test_add_duplicate_track_to_collection(
        self,
        client: TestClient,
        collection_with_tracks: Collection,
        public_track: Track,
        auth_headers: dict,
    ):
        response = client.post(
            f"/collections/{collection_with_tracks.id}/tracks",
            headers=auth_headers,
            json={"track_id": public_track.id},
        )
        assert response.status_code == 400
        assert "already in this collection" in response.json()["detail"]


class TestRemoveTrackFromCollection:
    def test_remove_track_from_collection_owner(
        self,
        client: TestClient,
        db_session: Session,
        collection_with_tracks: Collection,
        public_track: Track,
        auth_headers: dict,
    ):
        response = client.delete(
            f"/collections/{collection_with_tracks.id}/tracks/{public_track.id}",
            headers=auth_headers,
        )
        assert response.status_code == 204

        # Verify track is removed from collection
        ct = (
            db_session.query(CollectionTrack)
            .filter(
                CollectionTrack.collection_id == collection_with_tracks.id,
                CollectionTrack.track_id == public_track.id,
            )
            .first()
        )
        assert ct is None

    def test_remove_track_from_collection_other_user_forbidden(
        self,
        client: TestClient,
        collection_with_tracks: Collection,
        public_track: Track,
        other_auth_headers: dict,
    ):
        response = client.delete(
            f"/collections/{collection_with_tracks.id}/tracks/{public_track.id}",
            headers=other_auth_headers,
        )
        assert response.status_code == 403

    def test_remove_track_from_collection_admin(
        self,
        client: TestClient,
        db_session: Session,
        collection_with_tracks: Collection,
        public_track: Track,
        admin_auth_headers: dict,
    ):
        response = client.delete(
            f"/collections/{collection_with_tracks.id}/tracks/{public_track.id}",
            headers=admin_auth_headers,
        )
        assert response.status_code == 204

    def test_remove_track_from_collection_requires_auth(
        self,
        client: TestClient,
        collection_with_tracks: Collection,
        public_track: Track,
    ):
        response = client.delete(
            f"/collections/{collection_with_tracks.id}/tracks/{public_track.id}"
        )
        assert response.status_code == 401

    def test_remove_nonexistent_track_from_collection(
        self, client: TestClient, public_collection: Collection, auth_headers: dict
    ):
        response = client.delete(
            f"/collections/{public_collection.id}/tracks/99999",
            headers=auth_headers,
        )
        assert response.status_code == 404
        assert "Track not found in collection" in response.json()["detail"]


class TestUpdateTrackPosition:
    def test_update_track_position_owner(
        self,
        client: TestClient,
        collection_with_tracks: Collection,
        public_track: Track,
        auth_headers: dict,
    ):
        response = client.patch(
            f"/collections/{collection_with_tracks.id}/tracks/{public_track.id}",
            headers=auth_headers,
            json={"position": 10},
        )
        assert response.status_code == 200
        data = response.json()
        assert data["position"] == 10

    def test_update_track_position_other_user_forbidden(
        self,
        client: TestClient,
        collection_with_tracks: Collection,
        public_track: Track,
        other_auth_headers: dict,
    ):
        response = client.patch(
            f"/collections/{collection_with_tracks.id}/tracks/{public_track.id}",
            headers=other_auth_headers,
            json={"position": 10},
        )
        assert response.status_code == 403

    def test_update_track_position_requires_auth(
        self,
        client: TestClient,
        collection_with_tracks: Collection,
        public_track: Track,
    ):
        response = client.patch(
            f"/collections/{collection_with_tracks.id}/tracks/{public_track.id}",
            json={"position": 10},
        )
        assert response.status_code == 401

    def test_update_nonexistent_track_position(
        self, client: TestClient, public_collection: Collection, auth_headers: dict
    ):
        response = client.patch(
            f"/collections/{public_collection.id}/tracks/99999",
            headers=auth_headers,
            json={"position": 10},
        )
        assert response.status_code == 404


class TestReorderTracks:
    def test_reorder_tracks_owner(
        self,
        client: TestClient,
        collection_with_tracks: Collection,
        public_track: Track,
        private_track: Track,
        auth_headers: dict,
    ):
        # Reverse the order
        response = client.put(
            f"/collections/{collection_with_tracks.id}/tracks/reorder",
            headers=auth_headers,
            json={"track_ids": [private_track.id, public_track.id]},
        )
        assert response.status_code == 200
        data = response.json()
        assert len(data) == 2
        # Verify new positions
        track_positions = {item["track_id"]: item["position"] for item in data}
        assert track_positions[private_track.id] == 0
        assert track_positions[public_track.id] == 1

    def test_reorder_tracks_other_user_forbidden(
        self,
        client: TestClient,
        collection_with_tracks: Collection,
        public_track: Track,
        private_track: Track,
        other_auth_headers: dict,
    ):
        response = client.put(
            f"/collections/{collection_with_tracks.id}/tracks/reorder",
            headers=other_auth_headers,
            json={"track_ids": [private_track.id, public_track.id]},
        )
        assert response.status_code == 403

    def test_reorder_tracks_admin(
        self,
        client: TestClient,
        collection_with_tracks: Collection,
        public_track: Track,
        private_track: Track,
        admin_auth_headers: dict,
    ):
        response = client.put(
            f"/collections/{collection_with_tracks.id}/tracks/reorder",
            headers=admin_auth_headers,
            json={"track_ids": [private_track.id, public_track.id]},
        )
        assert response.status_code == 200

    def test_reorder_tracks_requires_auth(
        self,
        client: TestClient,
        collection_with_tracks: Collection,
        public_track: Track,
        private_track: Track,
    ):
        response = client.put(
            f"/collections/{collection_with_tracks.id}/tracks/reorder",
            json={"track_ids": [private_track.id, public_track.id]},
        )
        assert response.status_code == 401

    def test_reorder_tracks_nonexistent_collection(
        self, client: TestClient, auth_headers: dict
    ):
        response = client.put(
            "/collections/99999/tracks/reorder",
            headers=auth_headers,
            json={"track_ids": [1, 2]},
        )
        assert response.status_code == 404
