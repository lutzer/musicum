use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use libloading::{Library, Symbol};
use musicum_processor_sdk::{
    abi_stable::std_types::{RBox, RSlice, RSliceMut},
    analyzer::{AnalysisContext, AnalysisRequest, AnalysisResult, AudioAnalyser},
    ffi::{
        AbiAnalyzer_TO, AbiStreamProcessor_TO, AbiStructuralProcessor_TO,
        AnalysisContextFFI, AnalysisRequestFFI,
        AnalyzerDescriptorFFI, ProcessorDescriptorFFI, ProcessorEntry,
    },
    processor::{
        BaseProcessor, ProcessorContext, ProcessorDescriptor, Segment,
        StreamProcessor, StructuralProcessor,
    },
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
    create_fn:       unsafe extern "C" fn() -> ProcessorEntry,
    analyzer_desc:   Option<&'static AnalyzerDescriptorFFI>,
    analyzer_create: Option<unsafe extern "C" fn() -> AbiAnalyzer_TO<'static, RBox<()>>>,
}

pub struct LoadedProcessor {
    pub entry: ProcessorEntry,
    _lib: Arc<Library>,
}

impl LoadedProcessor {
    pub fn into_stream_processor(self) -> Option<Box<dyn StreamProcessor>> {
        match self.entry {
            ProcessorEntry::Stream(inner) =>
                Some(Box::new(FfiStreamProcessor { inner, _lib: self._lib })),
            _ => None,
        }
    }

    pub fn into_structural_processor(self) -> Option<Box<dyn StructuralProcessor>> {
        match self.entry {
            ProcessorEntry::Structural(inner) =>
                Some(Box::new(FfiStructuralProcessor { inner, _lib: self._lib })),
            _ => None,
        }
    }
}

pub struct FfiStreamProcessor {
    inner: AbiStreamProcessor_TO<'static, RBox<()>>,
    _lib:  Arc<Library>,
}

impl BaseProcessor for FfiStreamProcessor {
    fn init(&mut self, uuid: String, ctx: &ProcessorContext, analysis: &mut AnalysisContext) {
        let ffi_in = AnalysisContextFFI::from_context(analysis);
        let ffi_out = self.inner.init(uuid.into(), *ctx, ffi_in);
        ffi_out.drain_into(analysis);
    }
    fn descriptor(&self) -> &'static ProcessorDescriptor { unimplemented!() }
    fn get_parameter(&self, id: &str) -> f64 { self.inner.get_parameter(id.into()) }
    fn set_parameter(&mut self, id: &str, value: f64) { self.inner.set_parameter(id.into(), value); }
    fn requires_analysis(&self) -> bool { self.inner.requires_analysis() }
    fn get_analysis_hash(&self) -> String {
        self.inner.get_analysis_hash().into()
    }
}

impl StreamProcessor for FfiStreamProcessor {
    fn process(&mut self, buffer: &mut [f32], time: f64, ctx: &ProcessorContext) {
        self.inner.process(RSliceMut::from_mut_slice(buffer), time, *ctx);
    }
}

pub struct FfiStructuralProcessor {
    inner: AbiStructuralProcessor_TO<'static, RBox<()>>,
    _lib:  Arc<Library>,
}

impl BaseProcessor for FfiStructuralProcessor {
    fn init(&mut self, uuid: String, ctx: &ProcessorContext, analysis: &mut AnalysisContext) {
        let ffi_in = AnalysisContextFFI::from_context(analysis);
        let ffi_out = self.inner.init(uuid.into(), *ctx, ffi_in);
        ffi_out.drain_into(analysis);
    }
    fn descriptor(&self) -> &'static ProcessorDescriptor { unimplemented!() }
    fn get_parameter(&self, id: &str) -> f64 { self.inner.get_parameter(id.into()) }
    fn set_parameter(&mut self, id: &str, value: f64) { self.inner.set_parameter(id.into(), value); }
    fn requires_analysis(&self) -> bool { self.inner.requires_analysis() }
    fn get_analysis_hash(&self) -> String {
        self.inner.get_analysis_hash().into()
    }
}

impl StructuralProcessor for FfiStructuralProcessor {
    fn segments(&self, duration: f64, ctx: &ProcessorContext) -> Vec<Segment> {
        self.inner.segments(duration, *ctx).into_vec()
    }
}

pub struct FfiAnalyzer {
    inner: AbiAnalyzer_TO<'static, RBox<()>>,
    _lib:  Arc<Library>,
}

impl FfiAnalyzer {
    /// Returns the FFI-encoded analyzer output. Unlike the [`AudioAnalyser`]
    /// trait method (which deserializes via typetag and therefore only
    /// succeeds when the concrete `AnalysisResult` type is registered in the
    /// current binary's typetag inventory), this raw form keeps the result
    /// opaque to the host. Useful when the host needs to store results for
    /// later forwarding to a processor's `init()` without round-tripping
    /// through typetag.
    pub fn analyze_raw(
        &mut self,
        samples:   &[f32],
        time:      f64,
        exhausted: bool,
        context:   &ProcessorContext,
    ) -> Option<musicum_processor_sdk::ffi::AnalysisResultFFI> {
        match self.inner.analyze(
            RSlice::from_slice(samples),
            time,
            exhausted,
            *context,
        ) {
            musicum_processor_sdk::abi_stable::std_types::ROption::RSome(ffi) => Some(ffi),
            musicum_processor_sdk::abi_stable::std_types::ROption::RNone => None,
        }
    }
}

impl AudioAnalyser for FfiAnalyzer {
    fn init(&mut self, request: &AnalysisRequest) {
        let ffi = AnalysisRequestFFI::from(request);
        self.inner.init(ffi);
    }

    fn analyze(
        &mut self,
        samples:   &[f32],
        time:      f64,
        exhausted: bool,
        context:   &ProcessorContext,
    ) -> Option<(String, Box<dyn AnalysisResult>)> {
        let ffi = self.analyze_raw(samples, time, exhausted, context)?;
        let hash: String = ffi.hash.clone().into();
        let boxed = ffi.into_boxed()?;
        Some((hash, boxed))
    }
}

pub struct LoadedAnalyzer {
    pub analyzer: FfiAnalyzer,
    _lib: Arc<Library>,
}

pub struct ProcessorRegistry {
    entries: HashMap<String, RegistryEntry>,
}

impl ProcessorRegistry {
    pub fn new() -> Self {
        Self { entries: HashMap::new() }
    }

    /// Scans `dir` for dylibs. Files that don't export `musicum_processor_descriptor`
    /// are silently skipped. Returns an error only if a file exports the symbol
    /// but fails to load correctly (e.g. missing `musicum_processor_create`).
    pub fn load_dir(&mut self, dir: &Path) -> Result<(), ProcessorLoadError> {
        let ext = dylib_extension();
        let read_dir = std::fs::read_dir(dir).map_err(ProcessorLoadError::Io)?;

        for entry in read_dir {
            let path = entry.map_err(ProcessorLoadError::Io)?.path();
            if path.extension().and_then(|e| e.to_str()) != Some(ext) {
                continue;
            }
            self.try_load_file(&path)?;
        }
        Ok(())
    }

    fn try_load_file(&mut self, path: &Path) -> Result<(), ProcessorLoadError> {
        // Safety: loading arbitrary dylibs is inherently unsafe.
        let lib = unsafe {
            Library::new(path).map_err(|source| ProcessorLoadError::Load {
                path: path.to_owned(),
                source,
            })?
        };

        // Check for the descriptor symbol. If absent, not a processor plugin — skip.
        let descriptor: &'static ProcessorDescriptorFFI = unsafe {
            let sym: Result<Symbol<unsafe extern "C" fn() -> &'static ProcessorDescriptorFFI>, _> =
                lib.get(b"musicum_processor_descriptor\0");
            match sym {
                Err(_) => return Ok(()),
                Ok(f) => f(),
            }
        };

        let create_fn: unsafe extern "C" fn() -> ProcessorEntry = unsafe {
            let sym: Symbol<unsafe extern "C" fn() -> ProcessorEntry> =
                lib.get(b"musicum_processor_create\0").map_err(|_| {
                    ProcessorLoadError::SymbolNotFound {
                        path: path.to_owned(),
                        symbol: "musicum_processor_create",
                    }
                })?;
            *sym
        };

        let (analyzer_desc, analyzer_create) = unsafe {
            let desc_sym: Result<
                Symbol<unsafe extern "C" fn() -> &'static AnalyzerDescriptorFFI>,
                _,
            > = lib.get(b"musicum_analyzer_descriptor\0");
            match desc_sym {
                Err(_) => (None, None),
                Ok(desc_fn) => {
                    let create_sym: Symbol<
                        unsafe extern "C" fn() -> AbiAnalyzer_TO<'static, RBox<()>>,
                    > = lib.get(b"musicum_analyzer_create\0").map_err(|_| {
                        ProcessorLoadError::SymbolNotFound {
                            path: path.to_owned(),
                            symbol: "musicum_analyzer_create",
                        }
                    })?;
                    (Some(desc_fn()), Some(*create_sym))
                }
            }
        };

        let id = descriptor.id.as_str().to_owned();
        let lib = Arc::new(lib);
        self.entries.insert(id, RegistryEntry {
            descriptor,
            lib,
            create_fn,
            analyzer_desc,
            analyzer_create,
        });
        Ok(())
    }

    /// Instantiates a processor by its descriptor id.
    /// Can be called multiple times for the same id — each call returns an
    /// independent instance with its own state. All instances share the
    /// underlying `Arc<Library>` so the dylib stays loaded.
    pub fn create(&self, id: &str) -> Option<LoadedProcessor> {
        let entry = self.entries.get(id)?;
        // Safety: create_fn points into a still-loaded library (Arc keeps it alive).
        let processor_entry = unsafe { (entry.create_fn)() };
        Some(LoadedProcessor {
            entry: processor_entry,
            _lib: Arc::clone(&entry.lib),
        })
    }

    pub fn descriptors(&self) -> impl Iterator<Item = &ProcessorDescriptorFFI> {
        self.entries.values().map(|e| e.descriptor)
    }

    pub fn create_analyzer_for(&self, processor_id: &str) -> Option<LoadedAnalyzer> {
        let entry = self.entries.get(processor_id)?;
        let create_fn = entry.analyzer_create?;
        let inner = unsafe { create_fn() };
        Some(LoadedAnalyzer {
            analyzer: FfiAnalyzer {
                inner,
                _lib: Arc::clone(&entry.lib),
            },
            _lib: Arc::clone(&entry.lib),
        })
    }

    pub fn analyzer_descriptor(&self, processor_id: &str) -> Option<&AnalyzerDescriptorFFI> {
        self.entries.get(processor_id)?.analyzer_desc
    }
}

impl Default for ProcessorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

fn dylib_extension() -> &'static str {
    #[cfg(target_os = "macos")]   { "dylib" }
    #[cfg(target_os = "linux")]   { "so"    }
    #[cfg(target_os = "windows")] { "dll"   }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    { "so" }
}
