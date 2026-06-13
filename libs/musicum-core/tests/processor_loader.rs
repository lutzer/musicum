use std::path::PathBuf;

use musicum_core::processor_loader::{ProcessorLoadError, ProcessorRegistry};
use musicum_processor_sdk::ffi::ProcessorParamFFI;
use musicum_processor_sdk::processor::{BaseProcessor, ProcessorContext};

// ── helpers ───────────────────────────────────────────────────────────────────

fn dylib_ext() -> &'static str {
    if cfg!(target_os = "macos") { "dylib" }
    else if cfg!(target_os = "linux") { "so" }
    else { "dll" }
}

fn processors_dylib_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../musicum-processors/target/debug")
}

fn gain_dylib_path() -> PathBuf {
    processors_dylib_dir().join(format!("libgain.{}", dylib_ext()))
}

fn trim_dylib_path() -> PathBuf {
    processors_dylib_dir().join(format!("libtrim.{}", dylib_ext()))
}

fn ctx() -> ProcessorContext {
    ProcessorContext { playing: true, sample_rate: 44100, number_channels: 2 }
}

// ── gain: load / descriptor ───────────────────────────────────────────────────

#[test]
fn loads_gain_processor() {
    let path = gain_dylib_path();
    if !path.exists() {
        eprintln!("skipping: build gain first: cd libs/musicum-processors/gain && cargo build");
        return;
    }

    let mut registry = ProcessorRegistry::new();
    registry.load_dir(&processors_dylib_dir()).unwrap();

    let ids: Vec<_> = registry.descriptors().map(|d| d.id.as_str()).collect();
    assert!(
        ids.contains(&"gain_plugin"),
        "expected gain_plugin in registry, got: {ids:?}",
    );
}

#[test]
fn descriptor_has_correct_param_name() {
    let path = gain_dylib_path();
    if !path.exists() { return; }

    let mut registry = ProcessorRegistry::new();
    registry.load_dir(&processors_dylib_dir()).unwrap();

    let desc = registry.descriptors().find(|d| d.id.as_str() == "gain_plugin").unwrap();
    assert_eq!(desc.name.as_str(), "Gain");
    assert!(!desc.params.is_empty(), "expected at least one param");
}

#[test]
fn creates_multiple_independent_instances() {
    let path = gain_dylib_path();
    if !path.exists() { return; }

    let mut registry = ProcessorRegistry::new();
    registry.load_dir(&processors_dylib_dir()).unwrap();

    let mut a = registry.create("gain_plugin").unwrap();
    let b = registry.create("gain_plugin").unwrap();

    a.set_parameter("gain", 2.0);
    assert!((a.get_parameter("gain") - 2.0).abs() < 1e-6);
    assert!(
        (b.get_parameter("gain") - 1.0).abs() < 1e-6,
        "instance b should have default gain 1.0, got {}",
        b.get_parameter("gain")
    );
}

// ── gain: audio processing ────────────────────────────────────────────────────

#[test]
fn gain_at_2x_doubles_amplitude() {
    let path = gain_dylib_path();
    if !path.exists() { return; }

    let mut registry = ProcessorRegistry::new();
    registry.load_dir(&processors_dylib_dir()).unwrap();

    let mut p = registry.create("gain_plugin").unwrap();
    p.set_parameter("gain", 2.0);
    let mut samples = vec![0.5_f32; 64];
    p.process(&mut samples, 0.0, &ctx());
    for s in &samples {
        assert!((s - 1.0).abs() < 1e-6, "expected 1.0, got {s}");
    }
}

#[test]
fn gain_at_zero_silences_audio() {
    let path = gain_dylib_path();
    if !path.exists() { return; }

    let mut registry = ProcessorRegistry::new();
    registry.load_dir(&processors_dylib_dir()).unwrap();

    let mut p = registry.create("gain_plugin").unwrap();
    p.set_parameter("gain", 0.0);
    let mut samples = vec![1.0_f32; 64];
    p.process(&mut samples, 0.0, &ctx());
    for s in &samples {
        assert!(s.abs() < 1e-6, "expected 0.0, got {s}");
    }
}

#[test]
fn gain_default_is_unity() {
    let path = gain_dylib_path();
    if !path.exists() { return; }

    let mut registry = ProcessorRegistry::new();
    registry.load_dir(&processors_dylib_dir()).unwrap();

    let mut p = registry.create("gain_plugin").unwrap();
    let mut samples = vec![0.5_f32; 64];
    p.process(&mut samples, 0.0, &ctx());
    for s in &samples {
        assert!((s - 0.5).abs() < 1e-6, "expected 0.5 (unity gain), got {s}");
    }
}

// ── trim: load / descriptor ───────────────────────────────────────────────────

#[test]
fn loads_trim_processor() {
    let path = trim_dylib_path();
    if !path.exists() {
        eprintln!("skipping: build trim first: cd libs/musicum-processors/trim && cargo build");
        return;
    }

    let mut registry = ProcessorRegistry::new();
    registry.load_dir(&processors_dylib_dir()).unwrap();

    let ids: Vec<_> = registry.descriptors().map(|d| d.id.as_str()).collect();
    assert!(
        ids.contains(&"trim_processor"),
        "expected trim_processor in registry, got: {ids:?}",
    );
}

#[test]
fn trim_descriptor_has_start_and_end_params() {
    let path = trim_dylib_path();
    if !path.exists() { return; }

    let mut registry = ProcessorRegistry::new();
    registry.load_dir(&processors_dylib_dir()).unwrap();

    let desc = registry.descriptors().find(|d| d.id.as_str() == "trim_processor").unwrap();
    assert_eq!(desc.name.as_str(), "Trim");
    let param_ids: Vec<_> = desc.params.iter().map(|p| match p {
        ProcessorParamFFI::Time { id, .. } => id.as_str(),
        _ => "?",
    }).collect();
    assert!(param_ids.contains(&"start"), "expected start param, got {param_ids:?}");
    assert!(param_ids.contains(&"end"),   "expected end param, got {param_ids:?}");
}

// ── trim: parameters ──────────────────────────────────────────────────────────

#[test]
fn trim_set_get_parameter_roundtrip() {
    let path = trim_dylib_path();
    if !path.exists() { return; }

    let mut registry = ProcessorRegistry::new();
    registry.load_dir(&processors_dylib_dir()).unwrap();

    let mut p = registry.create("trim_processor").unwrap();
    p.set_parameter("start", 1.5);
    assert!((p.get_parameter("start") - 1.5).abs() < 1e-9);
    p.set_parameter("end", 2.5);
    assert!((p.get_parameter("end") - 2.5).abs() < 1e-9);
}

// ── trim: structural math ─────────────────────────────────────────────────────

#[test]
fn trim_segments_returns_single_segment() {
    let path = trim_dylib_path();
    if !path.exists() { return; }

    let mut registry = ProcessorRegistry::new();
    registry.load_dir(&processors_dylib_dir()).unwrap();

    let mut p = registry.create("trim_processor").unwrap();
    p.set_parameter("start", 1.0);
    p.set_parameter("end", 2.0);
    let segs = p.segments(10.0, &ctx());
    assert_eq!(segs.len(), 1);
    assert!((segs[0].src_start - 1.0).abs() < 1e-9);
    assert!((segs[0].src_end - 8.0).abs() < 1e-9);
    assert!((segs[0].rate - 1.0).abs() < 1e-9);
}

#[test]
fn trim_segments_empty_when_fully_trimmed() {
    let path = trim_dylib_path();
    if !path.exists() { return; }

    let mut registry = ProcessorRegistry::new();
    registry.load_dir(&processors_dylib_dir()).unwrap();

    let mut p = registry.create("trim_processor").unwrap();
    p.set_parameter("start", 6.0);
    p.set_parameter("end", 6.0);
    assert!(p.segments(10.0, &ctx()).is_empty());
}

// ── error cases ───────────────────────────────────────────────────────────────

#[test]
fn missing_directory_returns_io_error() {
    let mut registry = ProcessorRegistry::new();
    let result = registry.load_dir(std::path::Path::new("/nonexistent/path/xyz_musicum"));
    assert!(matches!(result, Err(ProcessorLoadError::Io(_))));
}

#[test]
fn empty_directory_loads_zero_processors() {
    let tmp = tempfile::tempdir().unwrap();
    let mut registry = ProcessorRegistry::new();
    registry.load_dir(tmp.path()).unwrap();
    assert_eq!(registry.descriptors().count(), 0);
}

#[test]
fn normalize_bundles_a_loadable_analyzer() {
    let path = processors_dylib_dir().join(format!("libnormalize.{}", dylib_ext()));
    if !path.exists() { return; }
    let mut registry = ProcessorRegistry::new();
    registry.load_dir(&processors_dylib_dir()).unwrap();
    assert!(registry.analyzer_descriptor("normalize").is_some());
    assert!(registry.create_analyzer_for("normalize").is_some());
}
