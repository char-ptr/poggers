use std::{ffi::c_void, mem::size_of, path::PathBuf, rc::Rc, sync::Arc};

use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::HANDLE,
        System::{
            LibraryLoader::GetModuleHandleW,
            Memory::{
                VirtualAlloc, VirtualFree, VirtualProtect, VirtualQuery, MEMORY_BASIC_INFORMATION,
                MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_PROTECTION_FLAGS,
            },
            ProcessStatus::{GetModuleInformation, GetProcessImageFileNameW, MODULEINFO},
            Threading::{GetCurrentProcess, GetCurrentProcessId},
        },
    },
};

use crate::{
    sigscan::SigScan,
    structures::{
        modules::{Module, ModuleError},
        process::{Internal, Process, ProcessBasics},
        protections::Protections,
    },
    traits::{Mem, MemError},
};

use super::super::utils::ProcessUtils;

impl Mem for Process<Internal> {
    unsafe fn alter_protection(
        &self,
        addr: usize,
        size: usize,
        prot: Protections,
    ) -> Result<Protections, MemError> {
        let mut old_prot = PAGE_PROTECTION_FLAGS::default();

        let prot_as_win = prot.native();

        let ok = VirtualProtect(addr as *const c_void, size, prot_as_win, &mut old_prot);
        if ok.is_ok() {
            Ok(old_prot.0.into())
        } else {
            Err(MemError::ProtectFailure(addr, size, prot))
        }
    }

    unsafe fn raw_read(&self, addr: usize, data: *mut u8, size: usize) -> Result<(), MemError> {
        (addr as *mut u8).copy_to_nonoverlapping(data, size);
        Ok(())
    }

    unsafe fn raw_write(&self, addr: usize, data: *const u8, size: usize) -> Result<(), MemError> {
        (addr as *mut u8).copy_from_nonoverlapping(data, size);
        Ok(())
    }

    unsafe fn raw_virtual_alloc(
        &self,
        addr: Option<usize>,
        size: usize,
        prot: Protections,
    ) -> Result<usize, MemError> {
        let alloc_ret = VirtualAlloc(
            addr.map(|x| x as *const c_void),
            size,
            MEM_COMMIT | MEM_RESERVE,
            prot.native(),
        );
        if alloc_ret.is_null() {
            Err(MemError::AllocFailure(addr, size))
        } else {
            Ok(alloc_ret as usize)
        }
    }
    unsafe fn raw_virtual_free(&self, addr: usize, size: usize) -> Result<(), MemError> {
        let is_ok = VirtualFree(addr as *mut c_void, size, MEM_RELEASE);
        if is_ok.is_ok() {
            Ok(())
        } else {
            Err(MemError::FreeFailure(addr, size))
        }
    }
    unsafe fn raw_query(&self, addr: usize) -> MEMORY_BASIC_INFORMATION {
        let mut info = MEMORY_BASIC_INFORMATION {
            RegionSize: 0x4096,
            ..Default::default()
        };
        VirtualQuery(
            Some(addr as *const c_void),
            &mut info,
            size_of::<MEMORY_BASIC_INFORMATION>(),
        );
        info
    }
}
impl Process<Internal> {
    /// constructs a process which is the current process
    pub(crate) fn new() -> Self {
        let handl = unsafe { GetCurrentProcess().0 };
        let proc_id = unsafe { GetCurrentProcessId() };
        Self {
            handl,
            pid: proc_id,
            mrk: Default::default(),
        }
    }
}
impl ProcessUtils for Process<Internal> {
    fn get_name(&self) -> String {
        let mut file_name = widestring::U16String::new();
        unsafe { GetProcessImageFileNameW(HANDLE(self.handl), file_name.as_mut_slice()) };
        file_name.to_string_lossy()
    }

    fn get_module(&self, name: &str) -> Result<Module<Self>, ModuleError>
    where
        Self: Sized + SigScan,
    {
        // let wstr = widestring::U16CString::from_str(name).unwrap();

        let module = unsafe { GetModuleHandleW(PCWSTR::null()) }
            .or(Err(ModuleError::NoModuleFound(name.to_string())))?;

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
        if !info.is_ok() {
            return Err(ModuleError::UnableToOpenHandle(name.to_string()));
        }
        Ok(Module {
            base_address: module.0 as usize,
            handle: module.0,
            end_address: (module.0 + mod_info.SizeOfImage as isize) as usize,
            name: Arc::new(name.to_string()),
            path: Arc::new(PathBuf::from(name)),
            owner: Arc::new(self.clone()),
            size: mod_info.SizeOfImage as usize,
        })
    }
}

impl SigScan for Process<Internal> {}
impl Clone for Process<Internal> {
    fn clone(&self) -> Self {
        Self {
            handl: self.get_handle().0,
            pid: self.pid,
            mrk: Default::default(),
        }
    }
}
