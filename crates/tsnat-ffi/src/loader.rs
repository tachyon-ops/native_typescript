use std::path::PathBuf;
use libloading::Library;
use std::collections::HashMap;
use std::sync::RwLock;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FfiError {
    #[error("Library load failed: {0}")]
    Load(#[from] libloading::Error),
    #[error("Library '{0}' not loaded")]
    NotLoaded(String),
    #[error("Failed to resolve symbol '{0}'")]
    SymbolNotFound(String),
}

pub struct NativeLibraryLoader {
    // Map module name (e.g. 'sdl3') to the loaded dynamic library.
    // Using simple String for keys to avoid requiring the Interner lock during lookups.
    libraries: RwLock<HashMap<String, Library>>,
}

impl NativeLibraryLoader {
    pub fn new() -> Self {
        Self {
            libraries: RwLock::new(HashMap::new()),
        }
    }

    /// Loads a shared library.
    /// In a real implementation this would use dlopen/LoadLibrary based on OS.
    /// libloading handles `.so` vs `.dll` vs `.dylib` appropriately depending on platform targets.
    pub fn load_library(&self, name: &str, path: PathBuf) -> Result<(), FfiError> {
        let lib = unsafe { Library::new(path)? };
        let mut libs = self.libraries.write().unwrap();
        libs.insert(name.to_string(), lib);
        Ok(())
    }

    /// Fetches a raw pointer to a symbol from ANY loaded library.
    /// Scans all loaded libraries if module is not strictly requested.
    /// For production, it may be scoped by library name. 
    pub fn resolve_symbol(&self, symbol_name: &str) -> Result<*mut std::ffi::c_void, FfiError> {
        let libs = self.libraries.read().unwrap();
        let c_str = std::ffi::CString::new(symbol_name).unwrap();

        for lib in libs.values() {
            unsafe {
                if let Ok(sym) = lib.get::<*mut std::ffi::c_void>(c_str.as_bytes_with_nul()) {
                    return Ok(*sym);
                }
            }
        }
        
        Err(FfiError::SymbolNotFound(symbol_name.to_string()))
    }
}
