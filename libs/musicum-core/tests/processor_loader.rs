use std::path::PathBuf;

use musicum_core::processor_loader::{ProcessorLoadError, ProcessorRegistry};
use musicum_processor_sdk::ffi::ProcessorEntry;

fn gain_dylib_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../musicum-processors/gain/target/debug")
}

fn gain_dylib_path() -> PathBuf {
    let ext = if cfg!(target_os = "macos") { "dylib" }
              else if cfg!(target_os = "linux") { "so" }
              else { "dll" };
    gain_dylib_dir().join(format!("libgain.{ext}"))
}

#[test]
fn loads_gain_processor() {
    let path = gain_dylib_path();
    if !path.exists() {
        eprintln!("skipping: build gain first: cd libs/musicum-processors/gain && cargo build");
        return;
    }

    let mut registry = ProcessorRegistry::new();
    registry.load_dir(&gain_dylib_dir()).unwrap();

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
    registry.load_dir(&gain_dylib_dir()).unwrap();

    let desc = registry.descriptors().find(|d| d.id.as_str() == "gain_plugin").unwrap();
    assert_eq!(desc.name.as_str(), "Gain");
    assert!(!desc.params.is_empty(), "expected at least one param");
}

#[test]
fn creates_multiple_independent_instances() {
    let path = gain_dylib_path();
    if !path.exists() { return; }

    let mut registry = ProcessorRegistry::new();
    registry.load_dir(&gain_dylib_dir()).unwrap();

    let mut a = registry.create("gain_plugin").unwrap();
    let mut b = registry.create("gain_plugin").unwrap();

    if let ProcessorEntry::Stream(ref mut p) = a.entry {
        p.set_parameter("gain".into(), 2.0);
        assert!((p.get_parameter("gain".into()) - 2.0).abs() < 1e-6);
    } else {
        panic!("expected Stream processor");
    }

    if let ProcessorEntry::Stream(ref mut p) = b.entry {
        assert!(
            (p.get_parameter("gain".into()) - 1.0).abs() < 1e-6,
            "instance b should have default gain 1.0, got {}",
            p.get_parameter("gain".into())
        );
    }
}

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
