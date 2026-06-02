use musicum_core::edit::{deserialize_processor_edits, EditKind, ProcessorEdit};
use std::collections::HashMap;
use uuid::Uuid;

#[test]
fn round_trips_structural_edit() {
    let mut params = HashMap::new();
    params.insert("start".to_string(), 1.0_f64);
    let edit = ProcessorEdit {
        uuid: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
        enabled: true,
        kind: EditKind::Structural { processor_id: "trim_processor".to_string(), params },
    };
    let json = serde_json::to_string(&vec![&edit]).unwrap();
    let loaded = deserialize_processor_edits(&json);
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0], edit);
}

#[test]
fn round_trips_stream_edit() {
    let mut params = HashMap::new();
    params.insert("gain".to_string(), 0.8_f64);
    let edit = ProcessorEdit {
        uuid: Uuid::parse_str("660e8400-e29b-41d4-a716-446655440001").unwrap(),
        enabled: false,
        kind: EditKind::Stream { processor_id: "gain_plugin".to_string(), params },
    };
    let json = serde_json::to_string(&vec![&edit]).unwrap();
    let loaded = deserialize_processor_edits(&json);
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0], edit);
}
