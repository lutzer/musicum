use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use libloading::{Library, Symbol};
use musicum_processor_sdk::ffi::{ProcessorDescriptorFFI, ProcessorEntry};

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
    descriptor: ProcessorDescriptorFFI,
    lib: Arc<Library>,
    create_fn: unsafe extern "C" fn() -> ProcessorEntry,
}

pub struct LoadedProcessor {
    pub entry: ProcessorEntry,
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
        let descriptor: ProcessorDescriptorFFI = unsafe {
            let sym: Result<Symbol<unsafe extern "C" fn() -> &'static ProcessorDescriptorFFI>, _> =
                lib.get(b"musicum_processor_descriptor\0");
            match sym {
                Err(_) => return Ok(()),
                Ok(f) => f().clone(),
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

        let id = descriptor.id.as_str().to_owned();
        let lib = Arc::new(lib);
        self.entries.insert(id, RegistryEntry { descriptor, lib, create_fn });
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
        self.entries.values().map(|e| &e.descriptor)
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
