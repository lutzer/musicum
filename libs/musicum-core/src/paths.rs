use std::path::Path;

/// Folder of `file_path` relative to `files_dir`, or `"-"` if it sits directly
/// under the library root or lies outside of it.
pub fn relative_folder(file_path: &str, files_dir: &Path) -> String {
    let path = Path::new(file_path);
    if let Some(parent) = path.parent() {
        if let Ok(rel) = parent.strip_prefix(files_dir) {
            let s = rel.to_string_lossy();
            if !s.is_empty() && s != "." {
                return s.to_string();
            }
        }
    }
    "-".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn nested_folder_returns_relative_path() {
        let files_dir = PathBuf::from("/lib");
        assert_eq!(
            relative_folder("/lib/drums/kicks/808.wav", &files_dir),
            "drums/kicks",
        );
    }

    #[test]
    fn file_at_root_returns_dash() {
        let files_dir = PathBuf::from("/lib");
        assert_eq!(relative_folder("/lib/intro.wav", &files_dir), "-");
    }

    #[test]
    fn file_outside_library_returns_dash() {
        let files_dir = PathBuf::from("/lib");
        assert_eq!(relative_folder("/elsewhere/song.wav", &files_dir), "-");
    }
}
