use std::path::PathBuf;
use musicum_core::processor_loader::ProcessorRegistry;
use musicum_processor_sdk::processor::{BaseProcessor, ProcessorContext};

fn processors_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..").join("musicum-processors").join("target").join("release")
}

#[test]
fn normalize_bundle_loads_processor_and_analyzer() {
    let dir = processors_dir();
    if !dir.exists() {
        eprintln!("skipping: build processors first");
        return;
    }
    let mut reg = ProcessorRegistry::new();
    reg.load_dir(&dir).expect("load_dir");

    let mut proc = reg.create("normalize").expect("normalize processor present");

    let p_ctx = ProcessorContext { playing: false, sample_rate: 44100, number_channels: 2 };
    proc.init("smoke-test".to_string(), &p_ctx);
    let request = proc.request_analysis(&p_ctx).expect("normalize requests analysis");

    let mut analyzer = reg.create_analyzer_for("normalize").expect("bundled analyzer present");
    analyzer.init(&request);
    // analyze_raw returns the FFI-encoded payload without typetag deserialization,
    // which would otherwise fail on the host (the concrete result type is only
    // registered in the dylib's typetag inventory).
    let raw = analyzer.analyze_raw(&[0.1, -0.2, 0.3, -0.4], 0.0, true, &p_ctx);
    let raw = raw.expect("analyzer produced a result");
    assert!(!raw.bytes.is_empty());
}

#[test]
fn standalone_processor_has_no_bundled_analyzer() {
    let dir = processors_dir();
    if !dir.exists() { return; }
    let mut reg = ProcessorRegistry::new();
    reg.load_dir(&dir).expect("load_dir");
    assert!(reg.create("level-meter").is_some());
    assert!(reg.create_analyzer_for("level-meter").is_none());
}
