/// implementation for modules
pub mod implement;
use std::{path::Path, sync::Arc};

use crate::sigscan::SigScan;
/// represents a module in a process
#[derive(Debug)]
pub struct Module<T: SigScan> {
    pub(crate) name: Arc<str>,
    pub(crate) path: Arc<Path>,
    pub(crate) base_address: usize,
    pub(crate) end_address: usize,
    pub(crate) size: usize,
    pub(crate) handle: isize,
    pub(crate) owner: Arc<T>,
}
#[cfg(windows)]
use windows::Win32::Foundation::HANDLE;

impl<T: SigScan> Module<T> {
    /// Get the name of the module
    pub fn get_name(&self) -> &str {
        &self.name
    }
    /// Get the base address of the module
    pub const fn get_base_address(&self) -> usize {
        self.base_address
    }
    /// Get the end address of the module
    pub const fn get_end_address(&self) -> usize {
        self.end_address
    }
    /// get path of the module
    pub fn get_path(&self) -> &Path {
        &self.path
    }
    /// Get the size of the module
    pub const fn get_size(&self) -> usize {
        self.size
    }
    #[cfg(not(windows))]
    /// Get the handle of the module
    pub const fn get_handle(&self) -> ! {
        panic!("Module handles are not supported on this platform")
    }
    #[cfg(windows)]
    /// Get the handle of the module
    pub const fn get_handle(&self) -> HANDLE {
        HANDLE(self.handle)
    }
    /// Get the owner of the module
    pub fn get_owner(&self) -> &T {
        self.owner.as_ref()
    }
}

/// Module errors
#[derive(Debug, thiserror::Error)]
pub enum ModuleError {
    /// The module was not found in the process.
    #[error("'{0}' was not found in the process")]
    NoModuleFound(String),
    /// The module handle could not be retrieved.
    #[error("unable to open handle for '{0}'")]
    UnableToOpenHandle(String),
}
