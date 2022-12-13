use std::{cell::RefCell, os::raw::c_void, rc::Rc, sync::Arc};

use windows::Win32::{
    Foundation::{CloseHandle, HANDLE, HINSTANCE},
    System::{
        Diagnostics::{
            Debug,
            ToolHelp::{
                CreateToolhelp32Snapshot, Module32First, Module32Next, MODULEENTRY32,
                TH32CS_SNAPMODULE, TH32CS_SNAPMODULE32,
            },
        },
        Memory::{VirtualQueryEx, MEMORY_BASIC_INFORMATION, MEM_COMMIT, PAGE_NOACCESS, VirtualQuery}, LibraryLoader::GetModuleHandleA, ProcessStatus::{K32GetModuleInformation, MODULEINFO}, Threading::GetCurrentProcess,
    },
};

use crate::mem::sigscan::SigScan;

use anyhow::{Context, Result};
use thiserror::Error;


/// A module in a process.
#[derive(Debug)]
pub struct InModule {
    pub(crate) base_address: usize,
    pub(crate) size: usize,
    pub(crate) name: String,
    pub(crate) handle: HINSTANCE,
}

impl InModule {
    /// create a new module object from a process and a module name.
    /// # Arguments
    /// * `name` - The name of the module to find.
    /// * `process` - The process to find the module in.
    /// # Example
    /// ```
    /// use poggers::mem::internal::process::Process;
    /// use poggers::mem::internal::module::InModule;
    /// let process = Process::new("notepad.exe").unwrap();
    /// let module = InModule::new("user32.dll", &process).unwrap();
    /// ```
    /// # Errors
    /// * [`ModuleError::NoModuleFound`] - The module was not found in the process.
    /// * [`ModuleError::UnableToOpenHandle`] - The module handle could not be retrieved.
    pub fn new(name: &str) -> Result<Self> {

        let lpc_str = windows::core::PCSTR::from_raw(format!("{}\n", name).as_ptr() as *const u8);

        let module = unsafe {GetModuleHandleA(lpc_str)}.or(Err(InModuleError::NoModuleFound(name.to_string())))?;

        let mut mod_info : MODULEINFO = Default::default();

        let proc = unsafe { GetCurrentProcess() } ; 

        let info = unsafe { K32GetModuleInformation(proc, module, &mut mod_info, std::mem::size_of::<MODULEINFO>() as u32) } ;

        if info == false {
            return Err(InModuleError::UnableToFetchInformation(name.to_string()).into());
        }

        Ok(Self {
            base_address: mod_info.EntryPoint as usize,
            size: mod_info.SizeOfImage as usize,
            name: name.to_string(),
            handle: module,
        })
    }

    /// Pattern scan this module to find an address
    /// # Arguments
    /// * `pattern` - The pattern to scan for (IDA Style).
    /// # Example
    /// ```
    /// use poggers::mem::internal::process::Process;
    /// use poggers::mem::internal::module::InModule;
    /// let process = Process::new("notepad.exe").unwrap();
    /// let module = InModule::new("user32.dll", &process).unwrap();
    /// let address = module.pattern_scan("48 8B 05 ? ? ? ? 48 8B 88 ? ? ? ? 48 85 C9 74 0A").unwrap();
    /// ```
    /// 
    pub fn scan_virtual(&self, pattern: &str) -> Option<usize> {
        let mut mem_info: MEMORY_BASIC_INFORMATION = Default::default();
        mem_info.RegionSize = 0x4096;

        println!("{} -> {}", self.base_address, self.size);

        let mut addr = self.base_address;

        loop { 
            if addr >= self.base_address + self.size {
                break;
            }

            let worky = unsafe {
                VirtualQuery(
                    addr as *const c_void,
                    &mut mem_info,
                    std::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
                )
            };
            if mem_info.State != MEM_COMMIT || mem_info.Protect == PAGE_NOACCESS {
                addr += mem_info.RegionSize as usize;
                continue;
            }

            let page = super::utils::read_sized(addr, mem_info.RegionSize - 1)
                .ok()?;

            let scan_res = self.scan_batch(pattern, &page);

            if let Some(result) = scan_res {
                println!("Found pattern at {:#x}", scan_res.unwrap());
                return Some(addr + result);
            }
            addr += mem_info.RegionSize as usize;
        }
        None
    }

    pub fn get_relative(&self, addr: usize) -> usize {
        addr - self.base_address
    }

    pub fn resolve_relative_ptr(&self, addr: usize, offset: u32) -> Result<usize> {
        let real_offset = super::utils::read::<u32>(addr + offset as usize)?;
        println!("Real offset: {:X?}", real_offset);
        Ok(self.base_address + (self.get_relative(addr) + real_offset as usize))
    }
}

#[derive(Debug, Error)]
pub enum InModuleError {
    #[error("Unable to open handle")]
    UnableToOpenHandle,
    #[error("No module found for {0}")]
    NoModuleFound(String),
    #[error("unable to get module information for {0}")]
    UnableToFetchInformation(String),
}

impl SigScan for InModule {
    unsafe fn read<T: Default>(&self, addr: usize) -> Result<T> {
        super::utils::read::<T>(addr)
    }
}