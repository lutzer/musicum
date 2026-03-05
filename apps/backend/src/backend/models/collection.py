from datetime import datetime

from sqlalchemy import DateTime, ForeignKey, Integer, String, Text, UniqueConstraint
from sqlalchemy.orm import Mapped, mapped_column, relationship

from backend.database import Base


class Collection(Base):
    __tablename__ = "collections"

    id: Mapped[int] = mapped_column(Integer, primary_key=True, index=True)
    name: Mapped[str] = mapped_column(String(255), index=True)
    description: Mapped[str | None] = mapped_column(Text, nullable=True)
    user_id: Mapped[int] = mapped_column(
        Integer, ForeignKey("users.id", ondelete="CASCADE"), index=True
    )
    is_public: Mapped[bool] = mapped_column(Integer, default=False)
    created_at: Mapped[datetime] = mapped_column(DateTime, default=datetime.utcnow)
    updated_at: Mapped[datetime] = mapped_column(
        DateTime, default=datetime.utcnow, onupdate=datetime.utcnow
    )

    user: Mapped["User"] = relationship("User", back_populates="collections")
    collection_tracks: Mapped[list["CollectionTrack"]] = relationship(
        "CollectionTrack", back_populates="collection", cascade="all, delete-orphan"
    )


class CollectionTrack(Base):
    __tablename__ = "collection_tracks"
    __table_args__ = (
        UniqueConstraint("collection_id", "track_id", name="uq_collection_track"),
    )

    id: Mapped[int] = mapped_column(Integer, primary_key=True, index=True)
    collection_id: Mapped[int] = mapped_column(
        Integer, ForeignKey("collections.id", ondelete="CASCADE"), index=True
    )
    track_id: Mapped[int] = mapped_column(
        Integer, ForeignKey("tracks.id", ondelete="CASCADE"), index=True
    )
    position: Mapped[int] = mapped_column(Integer, default=0)
    added_at: Mapped[datetime] = mapped_column(DateTime, default=datetime.utcnow)

    collection: Mapped["Collection"] = relationship(
        "Collection", back_populates="collection_tracks"
    )
    track: Mapped["Track"] = relationship("Track", back_populates="collection_tracks")


# Import at the end to avoid circular imports
from backend.models.track import Track  # noqa: E402, F401
from backend.models.user import User  # noqa: E402, F401
