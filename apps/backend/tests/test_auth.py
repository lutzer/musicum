from fastapi.testclient import TestClient

from backend.models.user import User


class TestRegister:
    def test_register_success(self, client: TestClient):
        response = client.post(
            "/auth/register",
            json={
                "username": "newuser",
                "email": "new@example.com",
                "password": "newpassword",
            },
        )
        assert response.status_code == 201
        data = response.json()
        assert data["username"] == "newuser"
        assert data["email"] == "new@example.com"
        assert data["role"] == "user"
        assert data["is_active"] == 1
        assert "id" in data
        assert "created_at" in data
        assert "updated_at" in data

    def test_register_duplicate_email(self, client: TestClient, test_user: User):
        response = client.post(
            "/auth/register",
            json={
                "username": "anotheruser",
                "email": test_user.email,
                "password": "somepassword",
            },
        )
        assert response.status_code == 400
        assert response.json()["detail"] == "Email already registered"

    def test_register_duplicate_username(self, client: TestClient, test_user: User):
        response = client.post(
            "/auth/register",
            json={
                "username": test_user.username,
                "email": "another@example.com",
                "password": "somepassword",
            },
        )
        assert response.status_code == 400
        assert response.json()["detail"] == "Username already taken"

    def test_register_invalid_email(self, client: TestClient):
        response = client.post(
            "/auth/register",
            json={
                "username": "invaliduser",
                "email": "notanemail",
                "password": "somepassword",
            },
        )
        assert response.status_code == 422


class TestLogin:
    def test_login_success(self, client: TestClient, test_user: User):
        response = client.post(
            "/auth/login",
            json={"email": "test@example.com", "password": "testpassword"},
        )
        assert response.status_code == 200
        data = response.json()
        assert "access_token" in data
        assert data["token_type"] == "bearer"

    def test_login_wrong_password(self, client: TestClient, test_user: User):
        response = client.post(
            "/auth/login",
            json={"email": "test@example.com", "password": "wrongpassword"},
        )
        assert response.status_code == 401
        assert response.json()["detail"] == "Incorrect email or password"

    def test_login_nonexistent_user(self, client: TestClient):
        response = client.post(
            "/auth/login",
            json={"email": "nonexistent@example.com", "password": "somepassword"},
        )
        assert response.status_code == 401
        assert response.json()["detail"] == "Incorrect email or password"


class TestGetMe:
    def test_get_me_success(
        self, client: TestClient, test_user: User, auth_headers: dict
    ):
        response = client.get("/auth/me", headers=auth_headers)
        assert response.status_code == 200
        data = response.json()
        assert data["email"] == test_user.email
        assert data["id"] == test_user.id

    def test_get_me_no_auth(self, client: TestClient):
        response = client.get("/auth/me")
        assert response.status_code == 401

    def test_get_me_invalid_token(self, client: TestClient):
        response = client.get(
            "/auth/me", headers={"Authorization": "Bearer invalidtoken"}
        )
        assert response.status_code == 401


class TestDeleteUser:
    def test_delete_own_account(
        self, client: TestClient, test_user: User, auth_headers: dict
    ):
        response = client.delete(f"/auth/users/{test_user.id}", headers=auth_headers)
        assert response.status_code == 204

    def test_delete_forbidden_for_others(
        self, client: TestClient, test_user: User, admin_user: User, auth_headers: dict
    ):
        response = client.delete(f"/auth/users/{admin_user.id}", headers=auth_headers)
        assert response.status_code == 403
        assert response.json()["detail"] == "Not authorized to delete this user"

    def test_admin_can_delete_any(
        self,
        client: TestClient,
        test_user: User,
        admin_user: User,
        admin_auth_headers: dict,
    ):
        response = client.delete(
            f"/auth/users/{test_user.id}", headers=admin_auth_headers
        )
        assert response.status_code == 204

    def test_delete_nonexistent_user(
        self, client: TestClient, admin_user: User, admin_auth_headers: dict
    ):
        response = client.delete("/auth/users/99999", headers=admin_auth_headers)
        assert response.status_code == 404
        assert response.json()["detail"] == "User not found"

    def test_delete_no_auth(self, client: TestClient, test_user: User):
        response = client.delete(f"/auth/users/{test_user.id}")
        assert response.status_code == 401


class TestAuthFlow:
    def test_full_user_lifecycle(self, client: TestClient):
        # 1. Register a new user
        register_response = client.post(
            "/auth/register",
            json={
                "username": "integrationuser",
                "email": "integration@example.com",
                "password": "securepassword123",
            },
        )
        assert register_response.status_code == 201
        user_data = register_response.json()
        user_id = user_data["id"]
        assert user_data["username"] == "integrationuser"
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
        assert me_data["username"] == "integrationuser"
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
        # Register a user
        client.post(
            "/auth/register",
            json={
                "username": "protecteduser",
                "email": "protected@example.com",
                "password": "password123",
            },
        )

        # Try to access /me without token
        response = client.get("/auth/me")
        assert response.status_code == 401

        # Try to delete without token
        response = client.delete("/auth/users/1")
        assert response.status_code == 401

    def test_cannot_delete_other_users(self, client: TestClient):
        # Register user 1
        user1_response = client.post(
            "/auth/register",
            json={
                "username": "user1",
                "email": "user1@example.com",
                "password": "password123",
            },
        )
        user1_id = user1_response.json()["id"]

        # Register user 2
        client.post(
            "/auth/register",
            json={
                "username": "user2",
                "email": "user2@example.com",
                "password": "password123",
            },
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
