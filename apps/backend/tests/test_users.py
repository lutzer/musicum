from fastapi.testclient import TestClient
from sqlalchemy.orm import Session

from backend.models.user import User, UserRole
from backend.services.auth_service import get_password_hash


class TestListUsers:
    def test_list_users_returns_active_users(self, client: TestClient, test_user: User):
        response = client.get("/users")
        assert response.status_code == 200
        data = response.json()
        assert data["total"] >= 1
        assert "items" in data
        assert "page" in data
        assert "page_size" in data

    def test_list_users_pagination(self, client: TestClient, db_session: Session):
        for i in range(5):
            user = User(
                username=f"paginationuser{i}",
                email=f"pagination{i}@example.com",
                hashed_password=get_password_hash("password"),
                role=UserRole.USER,
            )
            db_session.add(user)
        db_session.commit()

        response = client.get("/users?page=1&page_size=2")
        assert response.status_code == 200
        data = response.json()
        assert data["page"] == 1
        assert data["page_size"] == 2
        assert len(data["items"]) == 2
        assert data["total"] >= 5

    def test_list_users_excludes_inactive(
        self, client: TestClient, db_session: Session
    ):
        active_user = User(
            username="activeuser",
            email="active@example.com",
            hashed_password=get_password_hash("password"),
            role=UserRole.USER,
            is_active=1,
        )
        inactive_user = User(
            username="inactiveuser",
            email="inactive@example.com",
            hashed_password=get_password_hash("password"),
            role=UserRole.USER,
            is_active=0,
        )
        db_session.add_all([active_user, inactive_user])
        db_session.commit()

        response = client.get("/users")
        assert response.status_code == 200
        data = response.json()
        usernames = [user["username"] for user in data["items"]]
        assert "activeuser" in usernames
        assert "inactiveuser" not in usernames

    def test_list_users_response_format(self, client: TestClient, test_user: User):
        response = client.get("/users")
        assert response.status_code == 200
        data = response.json()
        assert len(data["items"]) > 0

        user_data = data["items"][0]
        assert "id" in user_data
        assert "username" in user_data
        assert "email" in user_data
        assert "role" in user_data
        assert "is_active" in user_data
        assert "created_at" in user_data
        assert "updated_at" in user_data
        assert "hashed_password" not in user_data

    def test_list_users_page_validation(self, client: TestClient):
        response = client.get("/users?page=0")
        assert response.status_code == 422

        response = client.get("/users?page_size=0")
        assert response.status_code == 422

        response = client.get("/users?page_size=101")
        assert response.status_code == 422

    def test_list_users_empty_result(self, client: TestClient, db_session: Session):
        db_session.query(User).delete()
        db_session.commit()

        response = client.get("/users")
        assert response.status_code == 200
        data = response.json()
        assert data["total"] == 0
        assert data["items"] == []
