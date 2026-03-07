from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    model_config = SettingsConfigDict(env_file=".env")

    DATABASE_URL: str = f"sqlite:///./data/musicum.db"
    SCHEMA_VERSION: str = "0.3"
    SECRET_KEY: str = "dev-secret-key-change-in-production"
    ALGORITHM: str = "HS256"
    ACCESS_TOKEN_EXPIRE_MINUTES: int = 30
    UPLOAD_DIR_TRACKS: str = "data/uploads/tracks"
    UPLOAD_DIR_ATTACHMENTS: str = "data/uploads/attachments"
    UPLOAD_DIR_CONVERTED: str = "data/uploads/converted"
    FFMPEG_PATH: str = "ffmpeg"
    DEV_MODE: bool = False


settings = Settings()
