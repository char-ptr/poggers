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
        Memory::{VirtualQueryEx, MEMORY_BASIC_INFORMATION, MEM_COMMIT, PAGE_NOACCESS},
    },
};

use crate::mem::sigscan::SigScan;

use super::process::ExProcess;
use anyhow::{Context, Result, anyhow};
use thiserror::Error;
use crate::mem::traits::Mem;

/// A module in a process.
#[derive(Debug)]
pub struct ExModule<'a> {
    pub(crate) process: &'a ExProcess,
    pub base_address: usize,
    pub size: usize,
    pub name: String,
    pub(crate) handle: HINSTANCE,
}

impl<'a> ExModule<'a> {
    /// create a new module object from a process and a module name.
    /// # Arguments
    /// * `name` - The name of the module to find.
    /// * `process` - The process to find the module in.
    /// # Example
    /// ```
    /// use poggers::mem::process::ExProcess;
    /// use poggers::mem::module::ExModule;
    /// let process = ExProcess::new("notepad.exe").unwrap();
    /// let module = ExModule::new("user32.dll", &process).unwrap();
    /// ```
    /// # Errors
    /// * [`ModuleError::NoModuleFound`] - The module was not found in the process.
    /// * [`ModuleError::UnableToOpenHandle`] - The module handle could not be retrieved.
    #[cfg(target_os = "windows")]
    pub fn new(name: &str, proc: &'a ExProcess) -> Result<Self> {
        use std::ffi::CString;

        let mut me: MODULEENTRY32 = Default::default();
        me.dwSize = std::mem::size_of::<MODULEENTRY32>() as u32;

        let snap_handl =
            unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, proc.pid) }
                .or(Err(ModuleError::UnableToOpenHandle))?;

        let mut result: Option<String> = None;

        let le_poggier = |m: &MODULEENTRY32| {
            let f = m.szModule.iter().map(|x| x.0).take_while(|x| *x != 0).collect::<Vec<u8>>();
            let x_name = String::from_utf8(f).unwrap();
            let x_name = x_name.trim_matches('\x00');
            return (x_name == name, x_name.to_string());
        };

        let hres = unsafe { Module32First(snap_handl, &mut me) };
        if !hres.as_bool() {
            return Err(ModuleError::UnableToOpenHandle.into());
        }
        let (is_poggier, mod_name) = le_poggier(&me);
        if is_poggier {
            result.replace(mod_name);
        } else {
            while unsafe { Module32Next(snap_handl, &mut me) }.as_bool()
                && result.is_none()
                && me.th32ProcessID != 0
            {
                let (is_ok, mod_name) = le_poggier(&me);
                if is_ok {
                    result = Some(mod_name);
                    break;
                } else {
                    continue;
                }
            }
        }
        if result.is_none() {
            return Err(ModuleError::NoModuleFound(name.to_string()).into());
        }
        Ok(Self {
            process: proc,
            base_address: me.modBaseAddr as usize,
            size: me.modBaseSize as usize,
            name: result.unwrap(),
            handle: me.hModule,
        })
    }

    /// Pattern scan this module to find an address
    /// # Arguments
    /// * `pattern` - The pattern to scan for (IDA Style).
    /// # Example
    /// ```
    /// use poggers::mem::process::ExProcess;
    /// use poggers::mem::module::ExModule;
    /// let process = ExProcess::new("notepad.exe").unwrap();
    /// let module = ExModule::new("user32.dll", &process).unwrap();
    /// let address = module.pattern_scan("48 8B 05 ? ? ? ? 48 8B 88 ? ? ? ? 48 85 C9 74 0A").unwrap();
    /// ```
    ///
    pub unsafe fn scan_virtual(&self, pattern: &str) -> Option<usize> {
        let mut mem_info: MEMORY_BASIC_INFORMATION = Default::default();
        mem_info.RegionSize = 0x4096;

        let mut addr = self.base_address;

        loop {
            if addr >= self.base_address + self.size {
                break;
            }

            let worky = unsafe {
                VirtualQueryEx(
                    self.process.handl,
                    Some(addr as *const c_void),
                    &mut mem_info,
                    std::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
                )
            };
            if mem_info.State != MEM_COMMIT || mem_info.Protect == PAGE_NOACCESS {
                addr += mem_info.RegionSize as usize;
                continue;
            }

            let page = self
                .read_sized(addr, mem_info.RegionSize - 1)
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

    /// Gets distance of address from base address.
    /// # Arguments
    /// * `addr` - The address to find the relative distance.
    /// * `offset` - Offset to add after address is solved.
    /// # Example
    /// ```
    /// use poggers::mem::internal::process::Process;
    /// use poggers::mem::internal::module::InModule;
    /// let module = InModule::new("ntdll.dll").unwrap();
    /// let relative = module.get_relative(0xDEADBEEF, 0x15);
    /// ```
    /// 
    pub fn get_relative(&self, addr: usize,offset:usize) -> usize {
        (addr - self.base_address) + offset
    }

    /// Gets pointer to data/address dynamically.
    /// # Arguments
    /// * `addr` - Address to run function with.
    /// * `offset` - Offset to add after address is solved.
    /// # Example
    /// ```
    /// use poggers::mem::internal::process::Process;
    /// use poggers::mem::internal::module::InModule;
    /// let module = InModule::new("ntdll.dll").unwrap();
    /// let actual_location = module.resolve_relative_ptr(0xDEADBEEF, 0x15);
    /// ```
    /// 
    pub unsafe fn resolve_relative_ptr(&self, addr: usize, offset: usize) -> Result<usize> {
        let real_offset = self.read::<u32>(addr)?;
        println!("Real offset: {:X?}", real_offset);
        let rel = self.get_relative(addr,offset);
        let real = rel + real_offset as usize;
        println!("Real: {:X?}", real);
        Ok(self.base_address + real)
        // Err(anyhow!("lazy"))
    }
}

#[derive(Debug, Error)]
pub enum ModuleError {
    #[error("Unable to open handle")]
    UnableToOpenHandle,
    #[error("No module found for {0}")]
    NoModuleFound(String),
}

impl<'a> SigScan for ExModule<'a> {}

impl<'a> Mem for ExModule<'a> {
    const READ_REQUIRE_PROTECTION: bool = true;
    unsafe fn alter_protection(&self,addr:usize, size: usize, prot: crate::mem::structures::Protections) -> Result<crate::mem::structures::Protections> {
        self.process.alter_protection(addr, size, prot)
    }
    unsafe fn raw_read(&self, addr: usize,data: *mut u8, size: usize) -> Result<()> {
        self.process.raw_read(addr, data, size)
    }
    unsafe fn raw_write(&self, addr: usize,data: *const u8, size: usize) -> Result<()> {
        self.process.raw_write(addr, data, size)
    }
}

// impl<'a> Drop for ExModule<'a> { // we don't own the handle
//     fn drop(&mut self) {
//         unsafe { CloseHandle(std::transmuteself.handle); }
//     }
// }
