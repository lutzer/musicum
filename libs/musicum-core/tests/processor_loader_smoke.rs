use std::path::PathBuf;
use musicum_core::processor_loader::ProcessorRegistry;
use musicum_processor_sdk::{
    analyzer::{AnalysisContext, AudioAnalyser},
    processor::ProcessorContext,
};

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

    let mut proc = reg.create("normalize")
        .expect("normalize processor present")
        .into_stream_processor()
        .expect("stream variant");

    let mut ctx = AnalysisContext::default();
    let p_ctx = ProcessorContext { playing: false, sample_rate: 44100, number_channels: 2 };
    <dyn musicum_processor_sdk::processor::StreamProcessor as
        musicum_processor_sdk::processor::BaseProcessor>::init(
            &mut *proc, "smoke-test".to_string(), &p_ctx, &mut ctx,
        );
    assert_eq!(ctx.requests.len(), 1);
    let request_hash = ctx.requests[0].hash.clone();

    let mut loaded = reg.create_analyzer_for("normalize")
        .expect("bundled analyzer present");
    loaded.analyzer.init(&ctx.requests[0]);
    // analyze_raw returns the FFI-encoded payload without typetag deserialization,
    // which would otherwise fail on the host (the concrete result type is only
    // registered in the dylib's typetag inventory).
    let raw = loaded.analyzer.analyze_raw(
        &[0.1, -0.2, 0.3, -0.4],
        0.0,
        true,
        &p_ctx,
    );
    let raw = raw.expect("analyzer produced a result");
    assert!(!raw.bytes.is_empty());
    assert_eq!(raw.hash.as_str(), request_hash.as_str());
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
