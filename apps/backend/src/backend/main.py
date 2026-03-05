from contextlib import asynccontextmanager

from fastapi import FastAPI

from backend.database import Base, engine
from backend.routers.auth import router as auth_router
from backend.routers.tracks import router as tracks_router


@asynccontextmanager
async def lifespan(app: FastAPI):
    Base.metadata.create_all(bind=engine)
    yield


app = FastAPI(
    title="Musicum API",
    description="API for organizing sound recordings and publishing collections",
    version="0.1.0",
    lifespan=lifespan,
)

app.include_router(auth_router)
app.include_router(tracks_router)


@app.get("/health")
def health_check():
    return {"status": "healthy"}
