import os
import subprocess
from pathlib import Path

from PIL import Image
from sqlalchemy import create_engine
from sqlalchemy.orm import sessionmaker

from backend.config import settings
from backend.models.track import ProcessingState, TrackAttachment


def process_image(
    input_path: str,
    output_path: str,
    max_dimension: int = 1920,
) -> bool:
    """Convert image to JPEG and resize to max dimension."""
    try:
        Path(output_path).parent.mkdir(parents=True, exist_ok=True)

        with Image.open(input_path) as img:
            if img.mode in ("RGBA", "P"):
                img = img.convert("RGB")

            width, height = img.size
            if width > max_dimension or height > max_dimension:
                if width > height:
                    new_width = max_dimension
                    new_height = int(height * (max_dimension / width))
                else:
                    new_height = max_dimension
                    new_width = int(width * (max_dimension / height))
                img = img.resize((new_width, new_height), Image.Resampling.LANCZOS)

            img.save(output_path, "JPEG", quality=85, optimize=True)
        return True
    except Exception:
        return False


def process_video(
    input_path: str,
    output_path: str,
    max_height: int = 1080,
) -> bool:
    """Convert video to MP4/H.264 and resize to max height."""
    try:
        Path(output_path).parent.mkdir(parents=True, exist_ok=True)

        cmd = [
            settings.FFMPEG_PATH,
            "-i",
            input_path,
            "-c:v",
            "libx264",
            "-preset",
            "medium",
            "-crf",
            "23",
            "-c:a",
            "aac",
            "-b:a",
            "128k",
            "-vf",
            f"scale=-2:'min({max_height},ih)'",
            "-movflags",
            "+faststart",
            "-y",
            output_path,
        ]
        result = subprocess.run(cmd, capture_output=True)
        return result.returncode == 0
    except Exception:
        return False


def process_attachment_background(
    attachment_id: int,
    input_path: str,
    output_path: str,
    attachment_type: str,
) -> None:
    """Background task to process attachment and update database."""
    try:
        engine = create_engine(
            settings.DATABASE_URL,
            connect_args={"check_same_thread": False},
        )
        session_local = sessionmaker(autocommit=False, autoflush=False, bind=engine)
        db = session_local()

        try:
            attachment = (
                db.query(TrackAttachment)
                .filter(TrackAttachment.id == attachment_id)
                .first()
            )
            if not attachment:
                return

            if attachment_type == "image":
                success = process_image(input_path, output_path)
            elif attachment_type == "video":
                success = process_video(input_path, output_path)
            else:
                return

            if success:
                attachment.processing_status = ProcessingState.READY
                # Delete original file and clear path (only keep processed file)
                if os.path.exists(input_path):
                    os.remove(input_path)
                attachment.path = output_path
            else:
                attachment.processing_status = ProcessingState.FAILED
                if os.path.exists(input_path):
                    os.remove(input_path)
                attachment.path = None
                

            db.commit()
        finally:
            db.close()
    except Exception:
        pass
