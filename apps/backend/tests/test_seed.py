from sqlalchemy.orm import Session

from backend.models.collection import Collection
from backend.models.track import Track
from backend.models.user import User
from backend.seed import (
    create_demo_user,
    create_sample_collection,
    create_sample_tracks,
    is_fresh_database,
    seed_database,
)


class TestSeedFunctions:
    def test_is_fresh_database_true_when_empty(self, db_session: Session):
        assert is_fresh_database(db_session) is True

    def test_is_fresh_database_false_when_user_exists(
        self, db_session: Session, test_user: User
    ):
        assert is_fresh_database(db_session) is False

    def test_create_demo_user(self, db_session: Session):
        user = create_demo_user(db_session)
        assert user.username == "demo"
        assert user.email == "demo@example.com"
        assert user.id is not None

    def test_create_sample_tracks(self, db_session: Session):
        user = create_demo_user(db_session)
        tracks = create_sample_tracks(db_session, user)
        assert len(tracks) == 5
        assert all(t.user_id == user.id for t in tracks)
        assert all(t.slug for t in tracks)

    def test_create_sample_collection(self, db_session: Session):
        user = create_demo_user(db_session)
        tracks = create_sample_tracks(db_session, user)
        collection = create_sample_collection(db_session, user, tracks)
        assert collection.name == "Ambient Favorites"
        assert collection.user_id == user.id
        assert len(collection.collection_tracks) > 0

    def test_seed_database_creates_all_data(self, db_session: Session):
        seed_database(db_session)

        users = db_session.query(User).all()
        assert len(users) == 1
        assert users[0].email == "demo@example.com"

        tracks = db_session.query(Track).all()
        assert len(tracks) == 5

        collections = db_session.query(Collection).all()
        assert len(collections) == 1

    def test_seed_database_skips_if_user_exists(
        self, db_session: Session, test_user: User
    ):
        initial_user_count = db_session.query(User).count()
        seed_database(db_session)
        final_user_count = db_session.query(User).count()
        assert initial_user_count == final_user_count
