import shutil
from pathlib import Path


def delete_file(filepath: str | None) -> None:
    if filepath is None:
        return
    path = Path(filepath)
    if path.exists():
        path.unlink()


def delete_dir(dirpath: str | None) -> None:
    if dirpath is None:
        return
    path = Path(dirpath)
    if path.exists():
        shutil.rmtree(path)
