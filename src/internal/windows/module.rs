use std::{ffi::CString, os::raw::c_void};

use windows::{
    core::{PCSTR, PCWSTR},
    Win32::{
        Foundation::HMODULE,
        System::{
            LibraryLoader::{GetModuleHandleW, GetProcAddress},
            Memory::{VirtualQuery, MEMORY_BASIC_INFORMATION, MEM_COMMIT, PAGE_NOACCESS, VirtualProtect, PAGE_PROTECTION_FLAGS, VirtualAlloc, MEM_RESERVE},
            ProcessStatus::{GetModuleInformation, MODULEINFO},
            Threading::GetCurrentProcess,
        },
    },
};

use crate::{sigscan::SigScan, traits::{Mem, MemError}, structures::VirtAlloc};

use thiserror::Error;


/// a util to read `size` bytes from current process memory
pub fn read_sized(addr:usize, size:usize) -> Result<Vec<u8>,InternalError> {
    let mut buffer = vec![0; size];
    let ptr = addr as *const u8;
    if ptr.is_null() {
        return Err(InternalError::InvalidPointer(addr).into())
    }

    unsafe {
        ptr.copy_to_nonoverlapping(buffer.as_mut_ptr(), size);
    }

    Ok(buffer)
}

/// Errors which may occur when reading/writing memory
#[derive(Debug, Error)]
pub enum InternalError {
    /// The pointer {0} is invalid
    #[error("'{0:X}' points to either an invalid address, or a null value")]
    InvalidPointer(usize),
}

/// A module in a process.
#[derive(Debug)]
pub struct InModule {
    /// The base address of the module.
    pub base_address: usize,
    /// The size of the module.
    pub size: usize,
    pub(crate) name: String,
    pub(crate) handle: HMODULE,
}

impl InModule {
    /// gets the module name
    pub fn get_name(&self) -> &str {
        &self.name
    }
    /// create a new module object from current process and a module name.
    /// # Arguments
    /// * `name` - The name of the module to find.
    /// # Example
    /// ```
    /// use poggers::internal::windows::module::InModule;
    /// let module = InModule::new("user32.dll").unwrap();
    /// ```
    /// # Errors
    /// * [`ModuleError::NoModuleFound`] - The module was not found in current process.
    /// * [`ModuleError::UnableToOpenHandle`] - The module handle could not be retrieved.
    pub fn new(name: &str) -> Result<Self,InModuleError> {
        let wstr = widestring::U16CString::from_str(name).unwrap();

        let module = unsafe { GetModuleHandleW(PCWSTR::from_raw(wstr.as_ptr())) }
            .or(Err(InModuleError::NoModuleFound(name.to_string())))?;

        let mut mod_info: MODULEINFO = Default::default();

        let proc = unsafe { GetCurrentProcess() };

        let info = unsafe {
            GetModuleInformation(
                proc,
                module,
                &mut mod_info,
                std::mem::size_of::<MODULEINFO>() as u32,
            )
        };

        if info == false {
            return Err(InModuleError::UnableToFetchInformation(name.to_string()));
        }

        Ok(Self {
            base_address: module.0 as usize,
            size: mod_info.SizeOfImage as usize,
            name: name.to_string(),
            handle: module,
        })
    }

    /// Gets exported function/procedure from a module.
    /// # Arguments
    /// * `name` - Name of the exported symbol.
    /// # Example
    /// ```
    /// use poggers::internal::windows::module::InModule;
    /// let module = InModule::new("ntdll.dll").unwrap();
    /// let nt_query_info = module.get_process_address("NtQuerySystemInformation").unwrap();
    /// ```
    ///
    pub fn get_process_address<T>(&self, name: &str) -> Option<T> {
        let wname = CString::new(name).unwrap();

        let result =
            unsafe { GetProcAddress(self.handle, PCSTR::from_raw(wname.as_ptr() as *const u8)) };
        result.map(|proc| unsafe { std::mem::transmute_copy(&proc) })

        // match unsafe { GetProcAddress(self.handle, lpc_name) } {
        //     Some(proc) => {
        //         proc
        //     },
        //     None => None
        // }
    }

    /// Pattern scan this module to find an address.
    /// # Arguments
    /// * `pattern` - The pattern to scan for (IDA Style).
    /// # Example
    /// ```
    /// use poggers::internal::windows::module::InModule;
    /// let module = InModule::new("user32.dll").unwrap();
    /// let address = module.scan_virtual("48 8B 05 ? ? ? ? 48 8B 88 ? ? ? ? 48 85 C9 74 0A").unwrap();
    /// ```
    ///
    pub fn scan_virtual(&self, pattern: &str) -> Option<usize> {
        let mut mem_info = MEMORY_BASIC_INFORMATION {
            RegionSize: 0x4096,
            ..Default::default()
        };

        println!("{} -> {}", self.base_address, self.size);

        let mut addr = self.base_address;

        loop {
            if addr >= self.base_address + self.size {
                break;
            }

            let _worky = unsafe {
                VirtualQuery(
                    Some(addr as *const c_void),
                    &mut mem_info,
                    std::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
                )
            };
            if mem_info.State != MEM_COMMIT || mem_info.Protect == PAGE_NOACCESS {
                addr += mem_info.RegionSize;
                continue;
            }

            let page = read_sized(addr, mem_info.RegionSize - 1).ok()?;

            let scan_res = self.scan(pattern, page.iter());

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
    /// # Example
    /// ```
    /// use poggers::internal::windows::module::InModule;
    /// let module = InModule::new("ntdll.dll").unwrap();
    /// let relative = module.get_relative(0xDEADBEEF, 0x15);
    /// ```
    ///
    pub fn get_relative(&self, addr: usize, offset: usize) -> usize {
        (addr - self.base_address) + offset
    }

    /// Gets pointer to data/address dynamically.
    /// # Arguments
    /// * `addr` - Address to run function with.
    /// * `offset` - Offset to add after address is solved.
    /// # Example
    /// ```
    /// use poggers::internal::windows::module::InModule;
    /// let module = InModule::new("ntdll.dll").unwrap();
    /// let actual_location = unsafe {module.resolve_relative_ptr(0xDEADBEEF, 0x15) };
    /// ```
    /// # Safety
    /// this is fine as long as address and offset lead to a valid address
    pub unsafe fn resolve_relative_ptr(&self, addr: usize, offset: usize) -> Result<usize,MemError> {
        let real_offset = self.read::<u32>(addr)?;
        println!("Real offset: {:X?}", real_offset);
        let rel = self.get_relative(addr, offset);
        let real = rel + real_offset as usize;
        println!("Real: {:X?}", real);
        Ok(self.base_address + real)
        // Err(anyhow!("lazy"))
    }
}

/// Errors that can occur when using InModule.
#[derive(Debug, Error)]
pub enum InModuleError {
    /// Unable to open handle
    #[error("Unable to open handle")]
    UnableToOpenHandle,
    /// No module found for {0}
    #[error("No module found for {0}")]
    NoModuleFound(String),
    /// unable to get module information for {0}
    #[error("unable to get module information for {0}")]
    UnableToFetchInformation(String),
    /// unable to allocate memory
    #[error("unable to allocate memory of size {0} to {1:X}")]
    UnableToAllocate(usize, usize),
}

impl Mem for InModule {
    unsafe fn alter_protection(&self,addr:usize, size: usize, prot: crate::structures::Protections) -> std::result::Result<crate::structures::Protections,crate::traits::MemError> {

        let mut old_prot = PAGE_PROTECTION_FLAGS::default();

        let prot_as_win = prot.native();

        let ok = VirtualProtect(addr as *const c_void, size, prot_as_win, &mut old_prot);

        if ok.as_bool() {
            Ok(prot)
        } else {
            Err(MemError::ProtectFailure(addr, size, prot))
        }
    }

    unsafe fn raw_read(&self, addr: usize,data: *mut u8, size: usize) -> std::result::Result<(),crate::traits::MemError> {
        (addr as *mut u8).copy_to_nonoverlapping(data, size);
        Ok(())
    }

    unsafe fn raw_write(&self, addr: usize,data: *const u8, size: usize) -> std::result::Result<(),crate::traits::MemError> {
        (addr as *mut u8).copy_from_nonoverlapping(data, size);
        Ok(())
    }

    unsafe fn virtual_alloc(&self, addr: usize, size: usize, prot: crate::structures::Protections) -> std::result::Result<crate::structures::VirtAlloc,crate::traits::MemError> {
        let alloc_ret = VirtualAlloc(
            Some(addr as *mut c_void),
            size,
            MEM_COMMIT | MEM_RESERVE,
            prot.native(),
        );
        if alloc_ret.is_null() {
            Err(MemError::AllocFailure(size, addr))
        } else {
            Ok(VirtAlloc {
                pid: 0,
                addr,
                size,
                intrn:true
            })
        }

    }
}
impl SigScan for InModule {}
