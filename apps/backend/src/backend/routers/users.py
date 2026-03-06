from fastapi import APIRouter, Depends, Query
from sqlalchemy.orm import Session

from backend.database import get_db
from backend.schemas.user import UserListResponse
from backend.services.user_service import get_users

router = APIRouter(prefix="/users", tags=["users"])


@router.get("", response_model=UserListResponse)
def list_users(
    page: int = Query(1, ge=1, description="Page number"),
    page_size: int = Query(20, ge=1, le=100, description="Items per page"),
    db: Session = Depends(get_db),
) -> dict:
    """List active users with pagination."""
    users, total = get_users(db, page=page, page_size=page_size)
    return {
        "items": users,
        "total": total,
        "page": page,
        "page_size": page_size,
    }
