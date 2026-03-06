from contextlib import asynccontextmanager

from fastapi import FastAPI

from backend.config import settings
from backend.database import Base, SessionLocal, engine
from backend.routers.auth import router as auth_router
from backend.routers.collections import router as collections_router
from backend.routers.tracks import router as tracks_router
from backend.routers.users import router as users_router
from backend.seed import is_fresh_database, seed_database


@asynccontextmanager
async def lifespan(app: FastAPI):
    Base.metadata.create_all(bind=engine)

    if settings.DEV_MODE:
        db = SessionLocal()
        try:
            if is_fresh_database(db):
                seed_database(db)
        finally:
            db.close()

    yield


app = FastAPI(
    title="Musicum API",
    description="API for organizing sound recordings and publishing collections",
    version="0.1.0",
    lifespan=lifespan,
)

app.include_router(auth_router)
app.include_router(tracks_router)
app.include_router(collections_router)
app.include_router(users_router)


@app.get("/health")
def health_check():
    return {"status": "healthy"}
