use crate::{structures::modules::{Module,ModuleError}, sigscan::SigScan};

/// utilities for processes
pub trait ProcessUtils {
    /// get a module by name
    fn get_module(&self, name:&str) -> Result<Module<Self>,ModuleError> where Self: Sized + SigScan;
}