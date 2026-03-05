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
