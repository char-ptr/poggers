use std::path::Path;
use std::{ffi::c_void, marker::PhantomData, mem::size_of, path::PathBuf, sync::Arc};
use tracing::instrument;
use windows::core::PWSTR;

use windows::Win32::System::Threading::{QueryFullProcessImageNameW, PROCESS_NAME_WIN32};
use windows::Win32::{
    Foundation::HANDLE,
    System::{
        Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory},
        Memory::{
            VirtualAllocEx, VirtualFreeEx, VirtualProtectEx, VirtualQueryEx,
            MEMORY_BASIC_INFORMATION, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE,
        },
        Threading::{OpenProcess, PROCESS_ALL_ACCESS},
    },
};

use crate::{
    sigscan::SigScan,
    structures::{
        create_snapshot::ToolSnapshot,
        modules::{Module, ModuleError},
        process::{External, Process, ProcessError, U32OrString},
        protections::Protections,
    },
    traits::{Mem, MemError},
};

use super::super::utils::ProcessUtils;
use super::WIN_PAGE_SIZE;

impl Mem for Process<External> {
    unsafe fn raw_query(&self, addr: usize) -> MEMORY_BASIC_INFORMATION {
        let mut info = MEMORY_BASIC_INFORMATION {
            RegionSize: WIN_PAGE_SIZE,
            ..Default::default()
        };
        VirtualQueryEx(
            HANDLE(self.handl),
            Some(addr as *const c_void),
            &mut info,
            size_of::<MEMORY_BASIC_INFORMATION>(),
        );
        info
    }
    unsafe fn alter_protection(
        &self,
        addr: usize,
        size: usize,
        prot: Protections,
    ) -> Result<Protections, MemError> {
        let mut old_protect = Default::default();
        unsafe {
            let Ok(_) = VirtualProtectEx(
                HANDLE(self.handl),
                addr as *const c_void,
                size,
                prot.native(),
                &mut old_protect,
            ) else {
                return Err(MemError::ProtectFailure(addr, size, prot));
            };
        }
        Ok(old_protect.0.into())
        // if res.as_bool() {
        //     Ok(old_protect.0.into())
        // } else {
        //     // plan to match in the future, cba atm
        //     let e = unsafe { GetLastError() };
        //     {
        //         println!("Error: {:?}", e);
        //     }
        //     Err(MemError::ProtectFailure(addr, size, prot))
        // }
    }
    unsafe fn raw_read(&self, addr: usize, data: *mut u8, size: usize) -> Result<(), MemError> {
        ReadProcessMemory(
            HANDLE(self.handl),
            addr as *const c_void,
            data as *mut _,
            size,
            Some(&mut 0),
        )
        .map_err(|_| MemError::ReadFailure(addr))
    }
    unsafe fn raw_write(&self, addr: usize, data: *const u8, size: usize) -> Result<(), MemError> {
        WriteProcessMemory(
            HANDLE(self.handl),
            addr as *const c_void,
            data as *const _,
            size,
            Some(&mut 0),
        )
        .map_err(|_| MemError::WriteFailure(addr))
    }
    unsafe fn raw_virtual_alloc(
        &self,
        addr: Option<usize>,
        size: usize,
        prot: Protections,
    ) -> Result<usize, MemError> {
        let alloc_ret = VirtualAllocEx(
            HANDLE(self.handl),
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
        VirtualFreeEx(HANDLE(self.handl), addr as *mut c_void, size, MEM_RELEASE)
            .map_err(|_| MemError::FreeFailure(addr, size))
    }
}
impl Process<External> {
    fn open_handle(process_id: u32) -> Result<HANDLE, ProcessError> {
        let hndl = unsafe {
            OpenProcess(PROCESS_ALL_ACCESS, false, process_id).or(Err(
                ProcessError::UnableToOpenProcess(U32OrString::U32(process_id)),
            ))?
        };
        if hndl.is_invalid() {
            Err(ProcessError::UnableToOpenProcess(U32OrString::U32(
                process_id,
            )))
        } else {
            Ok(hndl)
        }
    }

    fn get_name_from_hndl(hndl: HANDLE) -> String {
        let mut name_buf = [0u16; 256];
        let mut write_size = name_buf.len() as u32;
        let pwstr = PWSTR::from_raw(name_buf.as_mut_ptr());
        let _ =
            unsafe { QueryFullProcessImageNameW(hndl, PROCESS_NAME_WIN32, pwstr, &mut write_size) };
        let str = unsafe { pwstr.to_string().unwrap() };
        Path::new(&str)
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string()
    }
    /// finds the process from a pid
    pub fn find_by_pid(pid: u32) -> Result<Self, ProcessError> {
        let open_hndl = Self::open_handle(pid)?;
        Ok(Self {
            handl: open_hndl.0,
            pid,
            mrk: Default::default(),
        })
    }
    /// finds the process from a name
    pub fn find_by_name(name: &str) -> Result<Self, ProcessError> {
        // rename by or from to match possibly
        let mut snapshot = ToolSnapshot::new_process().unwrap();
        let res = snapshot.find(|process| process.exe_path == name).ok_or(
            ProcessError::UnableToFindProcess(U32OrString::String(name.to_string())),
        )?;
        Self::find_by_pid(res.id)
    }

    // pub function to allow people to make a process from an existing handle. really not very safe or recommended, but it's here if they're sure they want to use it
    /// get a process from a handle. assumes everything is valid about the handle (things could go very wrong if the permissions on handle is incorrect.)
    pub const fn from_handle(hnd: HANDLE, pid: u32) -> Self {
        Self {
            handl: hnd.0,
            pid,
            mrk: PhantomData,
        }
    }
}

//@TODO: (WINDOWS) need to update some fields.
impl ProcessUtils for Process<External> {
    #[instrument]
    fn get_module(&self, name: &str) -> Result<Module<Self>, ModuleError>
    where
        Self: Sized + SigScan,
    {
        let mut snapshot = ToolSnapshot::new_module(Some(self.pid)).unwrap();
        let res = snapshot
            .find(|module| module.name == name)
            .ok_or(ModuleError::NoModuleFound(name.to_string()))?;
        let owner = Arc::new(self.clone());

        Ok(Module {
            base_address: res.base_address,
            size: res.size,
            end_address: res.base_address + res.size,
            path: Arc::from(Path::new(&res.exe_path)),
            name: Arc::from(res.name.as_ref()),
            handle: res.handle.0,
            owner,
        })
    }
    #[instrument]
    fn get_name(&self) -> String {
        Self::get_name_from_hndl(HANDLE(self.handl))
    }
}
impl Clone for Process<External> {
    fn clone(&self) -> Self {
        Self {
            handl: self.handl,
            pid: self.pid,
            mrk: PhantomData,
        }
    }
}
