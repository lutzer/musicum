use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::edit::ProcessorEdit;

// ── Public types ──────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct ExportOptions {
    pub sample_rate:  Option<u32>,
    pub channels:     Option<u16>,
    pub bitrate_kbps: Option<u32>,
    pub overwrite:    bool,
}

#[derive(Debug)]
pub struct ExportResult {
    pub output_path:  PathBuf,
    pub format:       String,
    pub duration:     f64,
    pub sample_rate:  u32,
    pub channels:     u16,
    pub bitrate_kbps: Option<u32>,
}

// ── Main entry point ──────────────────────────────────────────────────────────

pub async fn export_audio(
    _file_path: &Path,
    _edits: &[ProcessorEdit],
    _output_path: &Path,
    _options: ExportOptions,
    _progress: impl Fn(f64, f64),
) -> Result<ExportResult> {
    todo!("export_audio: needs reimplementation with the new processor loader")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    const SUPPORTED_EXTS: &[&str] = &["wav", "mp3", "flac", "aiff", "aif"];

    fn is_lossless(ext: &str) -> bool {
        matches!(ext, "wav" | "flac" | "aiff" | "aif")
    }

    fn validate_extension(output_path: &Path) -> Result<String> {
        let ext = output_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        if SUPPORTED_EXTS.contains(&ext.as_str()) {
            Ok(ext)
        } else {
            anyhow::bail!("unsupported output format '{ext}'. Supported: wav, mp3, flac, aiff")
        }
    }

    #[test]
    fn validate_extension_rejects_unknown() {
        let err = validate_extension(Path::new("/out/file.ogg")).unwrap_err();
        assert!(err.to_string().contains("unsupported output format"));
    }

    #[test]
    fn validate_extension_accepts_all_supported() {
        for ext in SUPPORTED_EXTS {
            let path = Path::new("/out/file").with_extension(ext);
            assert!(validate_extension(&path).is_ok(), "should accept .{ext}");
        }
    }

    #[test]
    fn is_lossless_mp3_is_false() {
        assert!(!is_lossless("mp3"));
    }
}
