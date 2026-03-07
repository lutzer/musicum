from pathlib import Path

def delete_file(filepath: str):
    path = Path(filepath)
    if path.exists():
        path.unlink()

def delete_dir(dirpath: str):
    path = Path(dirpath)
    if path.exists():
        path.rmdir()