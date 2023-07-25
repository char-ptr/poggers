use std::os::raw::c_void;

use windows::Win32::{
    Foundation::HMODULE,
    System::Memory::{VirtualQueryEx, MEMORY_BASIC_INFORMATION, MEM_COMMIT, PAGE_NOACCESS},
};

use super::process::ExProcess;
use crate::{sigscan::SigScan, traits::MemError};
use crate::structures::Protections;
use crate::traits::Mem;
use anyhow::Result;
use thiserror::Error;

/// A module in a process.
#[derive(Debug)]
pub struct ExModule<'a> {
    pub(crate) process: &'a ExProcess,
    /// The base address of the module.
    pub base_address: usize,
    /// The size of the module.
    pub size: usize,
    /// The name of the module.
    pub name: String,
    pub(crate) handle: HMODULE,
}

impl<'a> ExModule<'a> {
    /// gets the module handle
    pub fn get_handle(&self) -> &HMODULE {
        &self.handle
    }
    /// create a new module object from a process and a module name.
    /// # Arguments
    /// * `name` - The name of the module to find.
    /// * `process` - The process to find the module in.
    /// # Example
    /// ```
    /// use poggers::external::module::ExModule;
    /// use poggers::external::process::ExProcess;
    /// let process = ExProcess::new("notepad.exe").unwrap();
    /// let module = ExModule::new("user32.dll", &process).unwrap();
    /// ```
    /// # Errors
    /// * [`ModuleError::NoModuleFound`] - The module was not found in the process.
    /// * [`ModuleError::UnableToOpenHandle`] - The module handle could not be retrieved.
    #[cfg(target_os = "windows")]
    pub fn new(name: &str, proc: &'a ExProcess) -> Result<Self> {
        use super::create_snapshot::ToolSnapshot;

        let mut snapshot = ToolSnapshot::new_module(Some(proc.pid)).unwrap();
        let res = snapshot
            .find(|module| module.name == name)
            .ok_or(ModuleError::NoModuleFound(name.to_string()))?;
        Ok(Self {
            process: proc,
            base_address: res.base_address,
            size: res.size,
            name: res.name,
            handle: res.handle,
        })
    }

    /// Pattern scan this module to find an address
    /// # Arguments
    /// * `pattern` - The pattern to scan for (IDA Style).
    /// # Example
    /// ```
    /// use poggers::external::module::ExModule;
    /// use poggers::external::process::ExProcess;
    /// let process = ExProcess::new("notepad.exe").unwrap();
    /// let module = ExModule::new("user32.dll", &process).unwrap();
    /// let address = module.pattern_scan("48 8B 05 ? ? ? ? 48 8B 88 ? ? ? ? 48 85 C9 74 0A").unwrap();
    /// ```
    /// # Safety
    /// This is safe to use as long as the pattern is correct.
    ///
    pub unsafe fn scan_virtual(&self, pattern: &str) -> Option<usize> {
        let mut mem_info = MEMORY_BASIC_INFORMATION {
            RegionSize: 0x4096,
            ..Default::default()
        };

        let mut addr = self.base_address;

        loop {
            if addr >= self.base_address + self.size {
                break;
            }

            let _worky = unsafe {
                VirtualQueryEx(
                    self.process.handl,
                    Some(addr as *const c_void),
                    &mut mem_info,
                    std::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
                )
            };
            if mem_info.State != MEM_COMMIT || mem_info.Protect == PAGE_NOACCESS {
                addr += mem_info.RegionSize;
                continue;
            }
            let mut page = [0u8; 0x4096];
            self.raw_read(addr, &mut page as *mut u8, 0x4096).ok()?;
            let scan_res = self.scan(pattern, page.iter());

            if let Some(result) = scan_res {
                println!("Found pattern at {:#x}", scan_res.unwrap());
                return Some(addr + result);
            }
            addr += 0x4096_usize;
        }
        None
    }
    /// scan pages for a value of <T>
    /// # Safety
    /// this should be completely safe to use
    pub unsafe fn scan_virtual_value<T>(&self, val: &T) -> Option<usize> {
        let mut mem_info = MEMORY_BASIC_INFORMATION {
            RegionSize: 0x4096,
            ..Default::default()
        };

        let mut addr = self.base_address;

        loop {
            if addr >= self.base_address + self.size {
                break;
            }

            let _worky = unsafe {
                VirtualQueryEx(
                    self.process.handl,
                    Some(addr as *const c_void),
                    &mut mem_info,
                    std::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
                )
            };
            if mem_info.Protect == PAGE_NOACCESS {
                addr += mem_info.RegionSize;
                continue;
            }

            let page = self.read_sized(addr, mem_info.RegionSize - 1).ok()?;
            let scan_res = self.scan_batch_value(val, &page);

            if let Some(result) = scan_res {
                println!("Found pattern at {:#x}", scan_res.unwrap());
                return Some(addr + result);
            }
            addr += mem_info.RegionSize;
        }
        None
    }

    /// Gets distance of address from base address.
    /// # Arguments
    /// * `addr` - The address to find the relative distance.
    /// * `offset` - Offset to add after address is solved.
    ///
    pub fn get_relative(&self, addr: usize, offset: usize) -> usize {
        (addr - self.base_address) + offset
    }

    /// Gets pointer to data/address dynamically.
    /// # Arguments
    /// * `addr` - Address to run function with.
    /// * `offset` - Offset to add after address is solved.
    /// # Safety
    /// this is safe to use aslong as address and offset are valid
    pub unsafe fn resolve_relative_ptr(&self, addr: usize, offset: usize) -> Result<usize> {
        let real_offset = self.read::<u32>(addr)?;
        println!("Real offset: {:X?}", real_offset);
        let rel = self.get_relative(addr, offset);
        let real = rel + real_offset as usize;
        println!("Real: {:X?}", real);
        Ok(self.base_address + real)
        // Err(anyhow!("lazy"))
    }
}
/// Errors which may occur when using a module.
#[derive(Debug, Error)]
pub enum ModuleError {
    /// Unable to open handle
    #[error("Unable to open handle")]
    UnableToOpenHandle,
    /// Unable to find handle for {0}
    #[error("No module found for {0}")]
    NoModuleFound(String),
}

impl<'a> SigScan for ExModule<'a> {}

impl<'a> Mem for ExModule<'a> {
    unsafe fn alter_protection(
        &self,
        addr: usize,
        size: usize,
        prot: Protections,
    ) -> Result<Protections, MemError> {
        self.process.alter_protection(addr, size, prot)
    }
    unsafe fn raw_read(&self, addr: usize, data: *mut u8, size: usize) -> Result<(),MemError> {
        self.process.raw_read(addr, data, size)
    }
    unsafe fn raw_write(&self, addr: usize, data: *const u8, size: usize) -> Result<(),MemError> {
        self.process.raw_write(addr, data, size)
    }
    unsafe fn virtual_alloc(
        &self,
        addr: usize,
        size: usize,
        prot: Protections,
    ) -> Result<crate::structures::VirtAlloc, MemError> {
        self.process.virtual_alloc(addr, size, prot)
    }
}

// impl<'a> Drop for ExModule<'a> { // we don't own the handle
//     fn drop(&mut self) {
//         unsafe { CloseHandle(std::transmuteself.handle); }
//     }
// }
