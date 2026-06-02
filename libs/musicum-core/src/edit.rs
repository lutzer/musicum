use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unified edit descriptor for both structural processors and audio plugins.
/// Stored in `ClipSidecar.processors` and passed to `PlaybackEngine`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProcessorEdit {
    pub uuid:    Uuid,
    pub enabled: bool,
    pub kind:    EditKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum EditKind {
    Structural {
        processor_id: String,
        #[serde(default)]
        params: HashMap<String, f64>,
    },
    #[serde(alias = "plugin")]
    Stream {
        processor_id: String,
        #[serde(default)]
        params: HashMap<String, f64>,
    },
}

pub fn deserialize_processor_edits(json: &str) -> Vec<ProcessorEdit> {
    serde_json::from_str(json).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_structural() -> ProcessorEdit {
        let mut params = HashMap::new();
        params.insert("start".to_string(), 1.0_f64);
        ProcessorEdit {
            uuid: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            enabled: true,
            kind: EditKind::Structural { processor_id: "trim".to_string(), params },
        }
    }

    fn make_stream() -> ProcessorEdit {
        let mut params = HashMap::new();
        params.insert("gain".to_string(), 0.5_f64);
        ProcessorEdit {
            uuid: Uuid::parse_str("660e8400-e29b-41d4-a716-446655440001").unwrap(),
            enabled: false,
            kind: EditKind::Stream { processor_id: "gain".to_string(), params },
        }
    }

    #[test]
    fn roundtrip_structural() {
        let edit = make_structural();
        let json = serde_json::to_string(&edit).unwrap();
        let back: ProcessorEdit = serde_json::from_str(&json).unwrap();
        assert_eq!(edit, back);
    }

    #[test]
    fn roundtrip_stream() {
        let edit = make_stream();
        let json = serde_json::to_string(&edit).unwrap();
        let back: ProcessorEdit = serde_json::from_str(&json).unwrap();
        assert_eq!(edit, back);
    }

    #[test]
    fn deserialize_new_format_vec() {
        let edits = vec![make_structural(), make_stream()];
        let json = serde_json::to_string(&edits).unwrap();
        let result = deserialize_processor_edits(&json);
        assert_eq!(result, edits);
    }

    #[test]
    fn deserialize_empty_json() {
        assert_eq!(deserialize_processor_edits("[]"), vec![]);
    }

    #[test]
    fn deserialize_garbage_returns_empty() {
        assert_eq!(deserialize_processor_edits("not json"), vec![]);
    }
}
