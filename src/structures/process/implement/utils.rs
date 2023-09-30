use crate::{
    sigscan::SigScan,
    structures::{
        modules::{Module, ModuleError},
        process::ProcessBasics,
    },
};

/// utilities for processes
pub trait ProcessUtils: ProcessBasics {
    /// get a module by name
    fn get_module(&self, name: &str) -> Result<Module<Self>, ModuleError>
    where
        Self: Sized + SigScan;
    /// get the base module, which is the module with the same name as the process
    fn get_base_module(&self) -> Result<Module<Self>, ModuleError>
    where
        Self: Sized + SigScan,
    {
        self.get_module(&self.get_name())
    }
    /// get the name of the process
    fn get_name(&self) -> String;
}
