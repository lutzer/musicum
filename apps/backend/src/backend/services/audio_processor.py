import subprocess
from pathlib import Path

from sqlalchemy import create_engine
from sqlalchemy.orm import sessionmaker

from backend.config import settings
from backend.models.track import Track

from pydub import AudioSegment

def convert_to_mp3(
    input_path: str,
    output_path: str,
    bitrate: str = "192k",
) -> bool:
    """Convert audio file to MP3 using FFmpeg."""
    Path(output_path).parent.mkdir(parents=True, exist_ok=True)

    cmd = [
        settings.FFMPEG_PATH,
        "-i",
        input_path,
        "-codec:a",
        "libmp3lame",
        "-b:a",
        bitrate,
        "-y",
        output_path,
    ]
    result = subprocess.run(cmd, capture_output=True)
    return result.returncode == 0

def get_duration(input_path: str) -> int | None:
    try:
        audio = AudioSegment.from_file(input_path)
        return int(len(audio) / 1000)
    except:
        return None

def process_track_background(
    track_id: int,
    input_path: str,
    output_path: str,
) -> None:
    """Background task to convert track and update database."""
    try:
        engine = create_engine(
            settings.DATABASE_URL,
            connect_args={"check_same_thread": False},
        )
        session_local = sessionmaker(autocommit=False, autoflush=False, bind=engine)
        db = session_local()

        try:
            track = db.query(Track).filter(Track.id == track_id).first()
            if not track:
                return

            success = convert_to_mp3(input_path, output_path)

            if success:
                track.processing_status = "ready"
                track.converted_path = output_path
            else:
                track.processing_status = "failed"

            duration = get_duration(input_path)
            if duration != None:
                track.duration_seconds = duration

            db.commit()
        finally:
            db.close()
    except Exception:
        pass
