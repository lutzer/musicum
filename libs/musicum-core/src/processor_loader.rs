use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use libloading::{Library, Symbol};
use musicum_processor_sdk::{
    abi_stable::std_types::{RBox, ROption, RSlice, RSliceMut},
    analyzer::{AnalysisContext, AnalysisRequest, AnalysisResult, AudioAnalyser},
    ffi::{
        AbiAnalyzer_TO, AbiProcessor_TO,
        AnalysisContextFFI, AnalysisRequestFFI, AnalysisResultFFI,
        ProcessorDescriptorFFI, ProcessorTypeFFI,
    },
    processor::{BaseProcessor, ProcessorContext, Segment},
};

pub enum ProcessorLoadError {
    Io(std::io::Error),
    SymbolNotFound { path: PathBuf, symbol: &'static str },
    Load { path: PathBuf, source: libloading::Error },
}

impl std::fmt::Display for ProcessorLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "io error: {e}"),
            Self::SymbolNotFound { path, symbol } =>
                write!(f, "{}: symbol `{symbol}` not found", path.display()),
            Self::Load { path, source } =>
                write!(f, "{}: failed to load: {source}", path.display()),
        }
    }
}

impl std::fmt::Debug for ProcessorLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

impl std::error::Error for ProcessorLoadError {}

struct RegistryEntry {
    descriptor:      &'static ProcessorDescriptorFFI,
    lib:             Arc<Library>,
    create_fn:       unsafe extern "C" fn() -> AbiProcessor_TO<'static, RBox<()>>,
    analyzer_create: Option<unsafe extern "C" fn() -> AbiAnalyzer_TO<'static, RBox<()>>>,
}

pub struct FfiProcessor {
    inner: AbiProcessor_TO<'static, RBox<()>>,
    _lib:  Arc<Library>,
}

impl BaseProcessor for FfiProcessor {
    fn init(&mut self, uuid: String, ctx: &ProcessorContext, analysis: &mut AnalysisContext) {
        let ffi_in  = AnalysisContextFFI::from_context(analysis);
        let ffi_out = self.inner.init(uuid.into(), *ctx, ffi_in);
        ffi_out.drain_into(analysis);
    }
    fn get_parameter(&self, id: &str) -> f64 { self.inner.get_parameter(id.into()) }
    fn set_parameter(&mut self, id: &str, value: f64) { self.inner.set_parameter(id.into(), value); }
    fn requires_analysis(&self) -> bool { self.inner.requires_analysis() }
    fn analysis_hash(&self) -> String { self.inner.get_analysis_hash().into() }
    fn process(&mut self, buffer: &mut [f32], time: f64, ctx: &ProcessorContext) {
        self.inner.process(RSliceMut::from_mut_slice(buffer), time, *ctx);
    }
    fn segments(&self, duration: f64, ctx: &ProcessorContext) -> Vec<Segment> {
        self.inner.segments(duration, *ctx).into_vec()
    }
}

pub struct FfiAnalyzer {
    inner: AbiAnalyzer_TO<'static, RBox<()>>,
    _lib:  Arc<Library>,
}

impl FfiAnalyzer {
    /// FFI-encoded result, skipping typetag deserialization (which only
    /// succeeds when the concrete `AnalysisResult` is in the host's typetag
    /// inventory).
    pub fn analyze_raw(
        &mut self, samples: &[f32], time: f64, exhausted: bool, context: &ProcessorContext,
    ) -> Option<AnalysisResultFFI> {
        match self.inner.analyze(RSlice::from_slice(samples), time, exhausted, *context) {
            ROption::RSome(ffi) => Some(ffi),
            ROption::RNone => None,
        }
    }
}

impl AudioAnalyser for FfiAnalyzer {
    fn init(&mut self, request: &AnalysisRequest) {
        self.inner.init(AnalysisRequestFFI::from(request));
    }
    fn analyze(
        &mut self, samples: &[f32], time: f64, exhausted: bool, context: &ProcessorContext,
    ) -> Option<(String, Box<dyn AnalysisResult>)> {
        let ffi = self.analyze_raw(samples, time, exhausted, context)?;
        let hash: String = ffi.hash.clone().into();
        Some((hash, ffi.into_boxed()?))
    }
}

pub struct ProcessorRegistry {
    entries: HashMap<String, RegistryEntry>,
}

impl ProcessorRegistry {
    pub fn new() -> Self { Self { entries: HashMap::new() } }

    /// Scans `dir` for dylibs. Files without `musicum_processor_descriptor`
    /// are silently skipped.
    pub fn load_dir(&mut self, dir: &Path) -> Result<(), ProcessorLoadError> {
        let ext = dylib_extension();
        for entry in std::fs::read_dir(dir).map_err(ProcessorLoadError::Io)? {
            let path = entry.map_err(ProcessorLoadError::Io)?.path();
            if path.extension().and_then(|e| e.to_str()) != Some(ext) { continue; }
            self.try_load_file(&path)?;
        }
        Ok(())
    }

    fn try_load_file(&mut self, path: &Path) -> Result<(), ProcessorLoadError> {
        let lib = unsafe {
            Library::new(path).map_err(|source| ProcessorLoadError::Load {
                path: path.to_owned(), source,
            })?
        };

        let descriptor: &'static ProcessorDescriptorFFI = unsafe {
            let sym: Result<Symbol<unsafe extern "C" fn() -> &'static ProcessorDescriptorFFI>, _> =
                lib.get(b"musicum_processor_descriptor\0");
            match sym {
                Err(_) => return Ok(()),
                Ok(f) => f(),
            }
        };

        let create_fn: unsafe extern "C" fn() -> AbiProcessor_TO<'static, RBox<()>> = unsafe {
            let sym: Symbol<unsafe extern "C" fn() -> AbiProcessor_TO<'static, RBox<()>>> =
                lib.get(b"musicum_processor_create\0").map_err(|_| {
                    ProcessorLoadError::SymbolNotFound {
                        path: path.to_owned(), symbol: "musicum_processor_create",
                    }
                })?;
            *sym
        };

        let analyzer_create: Option<unsafe extern "C" fn() -> AbiAnalyzer_TO<'static, RBox<()>>> = unsafe {
            let sym: Result<Symbol<unsafe extern "C" fn() -> AbiAnalyzer_TO<'static, RBox<()>>>, _> =
                lib.get(b"musicum_analyzer_create\0");
            sym.ok().map(|s| *s)
        };

        let id = descriptor.id.as_str().to_owned();
        self.entries.insert(id, RegistryEntry {
            descriptor, lib: Arc::new(lib), create_fn, analyzer_create,
        });
        Ok(())
    }

    /// Instantiates a processor by descriptor id; each call returns an
    /// independent instance sharing the same `Arc<Library>`.
    pub fn create(&self, id: &str) -> Option<FfiProcessor> {
        let entry = self.entries.get(id)?;
        let inner = unsafe { (entry.create_fn)() };
        Some(FfiProcessor { inner, _lib: Arc::clone(&entry.lib) })
    }

    pub fn descriptor(&self, id: &str) -> Option<&ProcessorDescriptorFFI> {
        self.entries.get(id).map(|e| e.descriptor)
    }

    pub fn descriptors(&self) -> impl Iterator<Item = &ProcessorDescriptorFFI> {
        self.entries.values().map(|e| e.descriptor)
    }

    pub fn processor_type(&self, id: &str) -> Option<ProcessorTypeFFI> {
        self.descriptor(id).map(|d| d.processor_type)
    }

    pub fn create_analyzer_for(&self, processor_id: &str) -> Option<FfiAnalyzer> {
        let entry = self.entries.get(processor_id)?;
        let create_fn = entry.analyzer_create?;
        let inner = unsafe { create_fn() };
        Some(FfiAnalyzer { inner, _lib: Arc::clone(&entry.lib) })
    }
}

impl Default for ProcessorRegistry {
    fn default() -> Self { Self::new() }
}

fn dylib_extension() -> &'static str {
    #[cfg(target_os = "macos")]   { "dylib" }
    #[cfg(target_os = "linux")]   { "so"    }
    #[cfg(target_os = "windows")] { "dll"   }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    { "so" }
}
