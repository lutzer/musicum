import os
import tempfile
from unittest.mock import MagicMock, patch

from backend.models.track import ProcessingState
from backend.services.media_processor import (
    process_attachment_background,
    process_image,
    process_video,
)


class TestProcessImage:
    def test_process_image_creates_jpeg(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            from PIL import Image

            input_path = os.path.join(tmpdir, "input.png")
            output_path = os.path.join(tmpdir, "output.jpg")

            img = Image.new("RGB", (100, 100), color="red")
            img.save(input_path, "PNG")

            result = process_image(input_path, output_path)

            assert result is True
            assert os.path.exists(output_path)

            with Image.open(output_path) as output_img:
                assert output_img.format == "JPEG"

    def test_process_image_resizes_large_image(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            from PIL import Image

            input_path = os.path.join(tmpdir, "input.png")
            output_path = os.path.join(tmpdir, "output.jpg")

            img = Image.new("RGB", (3000, 2000), color="blue")
            img.save(input_path, "PNG")

            result = process_image(input_path, output_path, max_dimension=1920)

            assert result is True
            assert os.path.exists(output_path)

            with Image.open(output_path) as output_img:
                assert max(output_img.size) == 1920

    def test_process_image_preserves_small_image_size(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            from PIL import Image

            input_path = os.path.join(tmpdir, "input.png")
            output_path = os.path.join(tmpdir, "output.jpg")

            img = Image.new("RGB", (800, 600), color="green")
            img.save(input_path, "PNG")

            result = process_image(input_path, output_path, max_dimension=1920)

            assert result is True
            assert os.path.exists(output_path)

            with Image.open(output_path) as output_img:
                assert output_img.size == (800, 600)

    def test_process_image_converts_rgba_to_rgb(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            from PIL import Image

            input_path = os.path.join(tmpdir, "input.png")
            output_path = os.path.join(tmpdir, "output.jpg")

            img = Image.new("RGBA", (100, 100), color=(255, 0, 0, 128))
            img.save(input_path, "PNG")

            result = process_image(input_path, output_path)

            assert result is True
            assert os.path.exists(output_path)

            with Image.open(output_path) as output_img:
                assert output_img.mode == "RGB"

    def test_process_image_returns_false_on_invalid_input(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            input_path = os.path.join(tmpdir, "nonexistent.png")
            output_path = os.path.join(tmpdir, "output.jpg")

            result = process_image(input_path, output_path)

            assert result is False

    def test_process_image_creates_output_directory(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            from PIL import Image

            input_path = os.path.join(tmpdir, "input.png")
            output_path = os.path.join(tmpdir, "subdir", "output.jpg")

            img = Image.new("RGB", (100, 100), color="red")
            img.save(input_path, "PNG")

            result = process_image(input_path, output_path)

            assert result is True
            assert os.path.exists(output_path)


class TestProcessVideo:
    @patch("backend.services.media_processor.subprocess.run")
    def test_process_video_calls_ffmpeg(self, mock_run):
        mock_run.return_value = MagicMock(returncode=0)

        with tempfile.TemporaryDirectory() as tmpdir:
            input_path = os.path.join(tmpdir, "input.mp4")
            output_path = os.path.join(tmpdir, "output.mp4")

            with open(input_path, "wb") as f:
                f.write(b"fake video content")

            result = process_video(input_path, output_path)

            assert result is True
            mock_run.assert_called_once()
            call_args = mock_run.call_args[0][0]
            assert "-c:v" in call_args
            assert "libx264" in call_args
            assert "-c:a" in call_args
            assert "aac" in call_args

    @patch("backend.services.media_processor.subprocess.run")
    def test_process_video_returns_false_on_failure(self, mock_run):
        mock_run.return_value = MagicMock(returncode=1)

        with tempfile.TemporaryDirectory() as tmpdir:
            input_path = os.path.join(tmpdir, "input.mp4")
            output_path = os.path.join(tmpdir, "output.mp4")

            with open(input_path, "wb") as f:
                f.write(b"fake video content")

            result = process_video(input_path, output_path)

            assert result is False


class TestProcessAttachmentBackground:
    @patch("backend.services.media_processor.process_image")
    @patch("backend.services.media_processor.create_engine")
    def test_process_attachment_background_image(
        self, mock_create_engine, mock_process_image
    ):
        mock_process_image.return_value = True

        mock_attachment = MagicMock()
        mock_attachment.id = 1
        mock_attachment.path = "/input.jpg"

        mock_session = MagicMock()
        mock_session.query.return_value.filter.return_value.first.return_value = (
            mock_attachment
        )

        mock_session_local = MagicMock(return_value=mock_session)
        with (
            patch(
                "backend.services.media_processor.sessionmaker",
                return_value=mock_session_local,
            ),
            tempfile.TemporaryDirectory() as tmpdir,
        ):
            input_path = os.path.join(tmpdir, "input.jpg")
            with open(input_path, "wb") as f:
                f.write(b"fake image data")

            process_attachment_background(
                attachment_id=1,
                input_path=input_path,
                output_path="/output.jpg",
                attachment_type="image",
            )

            # Verify original file is deleted
            assert not os.path.exists(input_path)

        mock_process_image.assert_called_once()
        assert mock_attachment.processing_status == ProcessingState.READY
        assert mock_attachment.path == "/output.jpg"
        mock_session.commit.assert_called_once()

    @patch("backend.services.media_processor.process_video")
    @patch("backend.services.media_processor.create_engine")
    def test_process_attachment_background_video(
        self, mock_create_engine, mock_process_video
    ):
        mock_process_video.return_value = True

        mock_attachment = MagicMock()
        mock_attachment.id = 1
        mock_attachment.path = "/input.mp4"

        mock_session = MagicMock()
        mock_session.query.return_value.filter.return_value.first.return_value = (
            mock_attachment
        )

        mock_session_local = MagicMock(return_value=mock_session)
        with (
            patch(
                "backend.services.media_processor.sessionmaker",
                return_value=mock_session_local,
            ),
            tempfile.TemporaryDirectory() as tmpdir,
        ):
            input_path = os.path.join(tmpdir, "input.mp4")
            with open(input_path, "wb") as f:
                f.write(b"fake video data")

            process_attachment_background(
                attachment_id=1,
                input_path=input_path,
                output_path="/output.mp4",
                attachment_type="video",
            )

            # Verify original file is deleted
            assert not os.path.exists(input_path)

        mock_process_video.assert_called_once()
        assert mock_attachment.processing_status == ProcessingState.READY
        assert mock_attachment.path == "/output.mp4"

    @patch("backend.services.media_processor.process_image")
    @patch("backend.services.media_processor.create_engine")
    def test_process_attachment_background_failure(
        self, mock_create_engine, mock_process_image
    ):
        mock_process_image.return_value = False

        mock_attachment = MagicMock()
        mock_attachment.id = 1

        mock_session = MagicMock()
        mock_session.query.return_value.filter.return_value.first.return_value = (
            mock_attachment
        )

        mock_session_local = MagicMock(return_value=mock_session)
        with patch(
            "backend.services.media_processor.sessionmaker",
            return_value=mock_session_local,
        ):
            process_attachment_background(
                attachment_id=1,
                input_path="/input.jpg",
                output_path="/output.jpg",
                attachment_type="image",
            )

        assert mock_attachment.processing_status == "failed"
