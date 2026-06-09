use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::edit::{ProcessorEdit};
use crate::ServiceError;
use crate::config;

// ── Legacy processor entry (kept for potential future migration paths) ───────
// These types are no longer part of the public API; use `ProcessorEdit` instead.

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct ProcessorRef {
    pub(crate) id:     String,
    pub(crate) params: serde_json::Value,
}

// ── Audio-file sidecar ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSidecar {
    #[serde(default)]
    pub id: String,
    pub version: u32,
    pub metadata: FileMetadataSidecar,
    #[serde(default)]
    pub attachments: Vec<AttachmentSidecar>,
    #[serde(default)]
    pub clips: Vec<ClipSidecar>,
}

impl FileSidecar {
    pub fn default_for_file() -> Self {
        FileSidecar {
            id: String::new(),
            version: 2,
            metadata: FileMetadataSidecar::default(),
            attachments: vec![],
            clips: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileMetadataSidecar {
    pub bpm: Option<f64>,
    pub key: Option<String>,
    pub rating: Option<i32>,
    pub color: Option<String>,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub tags: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentSidecar {
    pub uuid: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipSidecar {
    pub slug:  String,
    pub title: String,
    #[serde(default)]
    pub notes: String,
    /// Processor and plugin edits for this clip.
    #[serde(default, deserialize_with = "deserialize_clip_processors")]
    pub processors: Vec<ProcessorEdit>,
}

fn deserialize_clip_processors<'de, D>(d: D) -> Result<Vec<ProcessorEdit>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw = serde_json::Value::deserialize(d)?;
    serde_json::from_value(raw).map_err(serde::de::Error::custom)
}

// ── Read/write helpers ────────────────────────────────────────────────────

pub fn read_file_sidecar(audio_path: &Path) -> Result<FileSidecar, ServiceError> {
    let sidecar_path = sidecar_path_for_audio(audio_path);
    if !sidecar_path.exists() {
        return Ok(FileSidecar::default_for_file());
    }
    let text = std::fs::read_to_string(&sidecar_path)?;
    Ok(serde_json::from_str(&text)?)
}

pub fn write_file_sidecar(audio_path: &Path, sidecar: &FileSidecar) -> Result<(), ServiceError> {
    let sidecar_path = sidecar_path_for_audio(audio_path);
    let json = serde_json::to_string_pretty(sidecar)?;
    std::fs::write(&sidecar_path, json)?;
    Ok(())
}

pub fn sidecar_path_for_audio(audio_path: &Path) -> std::path::PathBuf {
    let hidden_sidecars = config::Config::get().general.hidden_sidecars;
    let stem = audio_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();
    audio_path
        .parent()
        .unwrap_or(Path::new("."))
        .join(if hidden_sidecars { format!(".{stem}.musicum.json") } else { format!("{stem}.musicum.json") })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use crate::edit::{ProcessorEdit, ProcessorEditType};
    use std::collections::HashMap;
    use uuid::Uuid;

    #[test]
    fn write_read_sidecar_with_processor_edits() {
        let dir = tempdir().unwrap();
        let audio = dir.path().join("test.wav");
        std::fs::write(&audio, b"").unwrap();

        let mut params = HashMap::new();
        params.insert("start".to_string(), 1.5_f64);
        let edit = ProcessorEdit {
            uuid: Uuid::new_v4(),
            enabled: true,
            processor_id: "trim".to_string(),
            kind: ProcessorEditType::StructuralProcessor,
            params,
        };

        let sc = FileSidecar {
            id: "test-file-id".to_string(),
            version: 1,
            metadata: FileMetadataSidecar::default(),
            attachments: vec![],
            clips: vec![ClipSidecar {
                slug: "c".to_string(),
                title: "C".to_string(),
                notes: String::new(),
                processors: vec![edit.clone()],
            }],
        };

        write_file_sidecar(&audio, &sc).unwrap();
        let loaded = read_file_sidecar(&audio).unwrap();
        assert_eq!(loaded.clips[0].processors[0], edit);
    }
}

