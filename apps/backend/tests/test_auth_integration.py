from fastapi.testclient import TestClient


class TestAuthFlow:
    def test_full_user_lifecycle(self, client: TestClient):
        """Test complete flow: register -> login -> authenticate -> delete."""
        # 1. Register a new user
        register_response = client.post(
            "/auth/register",
            json={"email": "integration@example.com", "password": "securepassword123"},
        )
        assert register_response.status_code == 201
        user_data = register_response.json()
        user_id = user_data["id"]
        assert user_data["email"] == "integration@example.com"

        # 2. Login with the new user
        login_response = client.post(
            "/auth/login",
            json={"email": "integration@example.com", "password": "securepassword123"},
        )
        assert login_response.status_code == 200
        token_data = login_response.json()
        assert "access_token" in token_data
        assert token_data["token_type"] == "bearer"
        access_token = token_data["access_token"]

        # 3. Use token to get current user info
        me_response = client.get(
            "/auth/me",
            headers={"Authorization": f"Bearer {access_token}"},
        )
        assert me_response.status_code == 200
        me_data = me_response.json()
        assert me_data["id"] == user_id
        assert me_data["email"] == "integration@example.com"

        # 4. Use token to delete the user
        delete_response = client.delete(
            f"/auth/users/{user_id}",
            headers={"Authorization": f"Bearer {access_token}"},
        )
        assert delete_response.status_code == 204

        # 5. Verify user is deleted (login should fail)
        login_after_delete = client.post(
            "/auth/login",
            json={"email": "integration@example.com", "password": "securepassword123"},
        )
        assert login_after_delete.status_code == 401

    def test_token_required_for_protected_endpoints(self, client: TestClient):
        """Test that protected endpoints require valid authentication."""
        # Register a user
        client.post(
            "/auth/register",
            json={"email": "protected@example.com", "password": "password123"},
        )

        # Try to access /me without token
        response = client.get("/auth/me")
        assert response.status_code == 401

        # Try to delete without token
        response = client.delete("/auth/users/1")
        assert response.status_code == 401

    def test_cannot_delete_other_users(self, client: TestClient):
        """Test that regular users cannot delete other users."""
        # Register user 1
        user1_response = client.post(
            "/auth/register",
            json={"email": "user1@example.com", "password": "password123"},
        )
        user1_id = user1_response.json()["id"]

        # Register user 2
        client.post(
            "/auth/register",
            json={"email": "user2@example.com", "password": "password123"},
        )

        # Login as user 2
        login_response = client.post(
            "/auth/login",
            json={"email": "user2@example.com", "password": "password123"},
        )
        user2_token = login_response.json()["access_token"]

        # Try to delete user 1 as user 2 (should fail)
        delete_response = client.delete(
            f"/auth/users/{user1_id}",
            headers={"Authorization": f"Bearer {user2_token}"},
        )
        assert delete_response.status_code == 403

        # Verify user 1 still exists (can still login)
        login_user1 = client.post(
            "/auth/login",
            json={"email": "user1@example.com", "password": "password123"},
        )
        assert login_user1.status_code == 200
