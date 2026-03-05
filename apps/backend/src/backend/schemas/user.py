from datetime import datetime

from pydantic import BaseModel, ConfigDict, EmailStr

from backend.models.user import UserRole


class UserCreate(BaseModel):
    email: EmailStr
    password: str


class UserResponse(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    id: int
    email: str
    role: UserRole
    is_active: int
    created_at: datetime
    updated_at: datetime
