from sqlalchemy.orm import Session

from backend.models.user import User, UserRole
from backend.schemas.user import UserCreate
from backend.services.auth_service import get_password_hash


def get_user_by_email(db: Session, email: str) -> User | None:
    return db.query(User).filter(User.email == email).first()


def create_user(
    db: Session, user_data: UserCreate, role: UserRole = UserRole.USER
) -> User:
    hashed_password = get_password_hash(user_data.password)
    user = User(
        email=user_data.email,
        hashed_password=hashed_password,
        role=role,
    )
    db.add(user)
    db.commit()
    db.refresh(user)
    return user


def delete_user(db: Session, user: User) -> None:
    db.delete(user)
    db.commit()
