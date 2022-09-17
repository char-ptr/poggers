use anyhow::{Context, Result};
use std::{error::Error, ffi::c_void, fmt::Display};
use thiserror::Error;

use windows::Win32::{
    Foundation::{CloseHandle, GetLastError, HANDLE, HINSTANCE, MAX_PATH},
    System::{
        Diagnostics::{
            Debug::{ReadProcessMemory, WriteProcessMemory},
            ToolHelp::{
                CreateToolhelp32Snapshot, Process32First, Process32Next,
                Toolhelp32ReadProcessMemory, PROCESSENTRY32, TH32CS_SNAPPROCESS,
            },
        },
        LibraryLoader::GetModuleFileNameW,
        Memory::{
            VirtualProtectEx, PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS, PAGE_READONLY,
            PAGE_READWRITE,
        },
        ProcessStatus::K32GetModuleFileNameExW,
        Threading::{OpenProcess, PROCESS_ALL_ACCESS, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ},
    },
};

use super::module::Module;
#[derive(Debug)]
pub struct Process {
    pub(crate) handl: windows::Win32::Foundation::HANDLE,
    pub(crate) pid: u32,
    pub(crate) name: String,
}

impl<'a> Process {
    pub fn new_from_pid(pid: u32) -> Result<Process> {
        let process_name = Self::get_name_from_pid(pid)?;
        let handle = Self::open_handle(pid)?;

        Ok(Process {
            handl: handle,
            pid,
            name: process_name,
        })
    }
    pub fn new_from_name(name: String) -> Result<Process> {
        let pid = Self::get_pid_from_name(&name)?;
        let handle = Self::open_handle(pid)?;

        Ok(Process {
            handl: handle,
            pid,
            name,
        })
    }
    fn get_pid_from_name(proc_name: &str) -> Result<u32> {
        let mut pe: PROCESSENTRY32 = Default::default();
        pe.dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;

        let hsnapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }.or(Err(
            ProcessError::UnableToOpenProcess(StringOru32::String(proc_name.to_string())),
        ))?;

        let mut result: Option<u32> = None;
        let le_poggier = |p: &PROCESSENTRY32| {
            let f = p.szExeFile.iter().map(|x| x.0).collect::<Vec<u8>>();
            let name = String::from_utf8_lossy(&f);
            return name == proc_name;
        };

        let hres = unsafe { Process32First(hsnapshot, &mut pe) };
        if !hres.as_bool() {
            return Err(ProcessError::UnableToOpenProcess(StringOru32::String(
                proc_name.to_string(),
            ))
            .into());
        }
        if le_poggier(&pe) {
            result.replace(pe.th32ProcessID);
        }
        while unsafe { Process32Next(hsnapshot, &mut pe) }.as_bool()
            && result.is_none()
            && pe.th32ProcessID != 0
        {
            if le_poggier(&pe) {
                result.replace(pe.th32ProcessID);
                break;
            } else {
                continue;
            }
        }
        unsafe {
            CloseHandle(hsnapshot);
        }
        result
            .ok_or(ProcessError::NoProcessFound(StringOru32::String(proc_name.to_string())).into())
    }
    fn get_name_from_pid(process_id: u32) -> Result<String> {
        if process_id == 0 {
            return Err(ProcessError::InvalidPid(process_id).into());
        }
        let hndl = unsafe {
            OpenProcess(
                PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
                false,
                process_id,
            )
            .or(Err(ProcessError::UnableToOpenProcess(StringOru32::U32(
                process_id,
            ))))?
        };
        if hndl.is_invalid() {
            return Err(ProcessError::UnableToOpenProcess(StringOru32::U32(process_id)).into());
        }
        let mut buffer = [0u16; MAX_PATH as usize];
        unsafe { K32GetModuleFileNameExW(hndl, None, &mut buffer) };
        // println!("{:?}", buffer);
        Ok(String::from_utf16_lossy(&buffer)
            .rsplit("\\")
            .next()
            .unwrap()
            .trim_matches('\x00')
            .to_string())
    }

    fn open_handle(process_id: u32) -> Result<HANDLE> {
        let hndl = unsafe {
            OpenProcess(PROCESS_ALL_ACCESS, false, process_id).or(Err(
                ProcessError::UnableToOpenProcess(StringOru32::U32(process_id)),
            ))?
        };
        if hndl.is_invalid() {
            Err(ProcessError::UnableToOpenProcess(StringOru32::U32(process_id)).into())
        } else {
            Ok(hndl)
        }
    }

    pub fn solve_dma(&self, addr: usize, offsets: &Vec<usize>) -> Result<usize> {
        let mut ptr = addr;
        for offset in offsets {
            ptr = self.read::<usize>(addr)?;
            ptr += offset;
        }
        Ok(ptr)
    }

    pub fn read<T: Default>(&self, addr: usize) -> Result<T> {
        let t_size = std::mem::size_of::<T>();
        let mut buffer: T = Default::default();

        let old_prot = self.change_protection(addr, t_size, PAGE_EXECUTE_READWRITE)?;

        let res = unsafe {
            ReadProcessMemory(
                self.handl,
                addr as *const c_void,
                &mut buffer as *mut _ as *mut _,
                t_size,
                &mut 0,
            )
        };

        let _ = self.change_protection(addr, t_size, old_prot);

        if res.as_bool() {
            Ok(buffer)
        } else {
            Err(ProcessError::UnableToReadMemory(addr as usize).into())
        }
    }
    pub fn read_sized(&self, addr: usize, size: usize) -> Result<Vec<u8>> {
        let mut buffer = vec![0; size];

        let old_prot = self.change_protection(addr, size, PAGE_EXECUTE_READWRITE)?;
        let res = unsafe {
            ReadProcessMemory(
                self.handl,
                addr as *const c_void,
                buffer.as_mut_ptr() as *mut _ as *mut _,
                size,
                &mut 0,
            )
        };

        let _ = self.change_protection(addr, size, old_prot)?;

        if res.as_bool() {
            Ok(buffer)
        } else {
            Err(ProcessError::UnableToReadMemory(addr as usize).into())
        }
    }

    pub fn write<T>(&self, addr: usize, data: &T) -> Result<()> {
        let old_prot = self.change_protection(addr, std::mem::size_of::<T>(), PAGE_READWRITE)?;
        let d = unsafe {
            let d = std::mem::transmute::<&T, *const c_void>(data);
            WriteProcessMemory(
                self.handl,
                addr as *const c_void,
                d,
                std::mem::size_of::<T>(),
                &mut 0,
            )
        };

        let _ = self.change_protection(addr, std::mem::size_of::<T>(), old_prot);

        if d.as_bool() {
            Ok(())
        } else {
            Err(ProcessError::UnableToWriteMemory(addr as usize).into())
        }
    }
    pub fn change_protection(
        &self,
        addr: usize,
        size: usize,
        protection: PAGE_PROTECTION_FLAGS,
    ) -> Result<PAGE_PROTECTION_FLAGS> {
        let mut old_protect = Default::default();
        let res = unsafe {
            VirtualProtectEx(
                self.handl,
                addr as *const c_void,
                size,
                protection,
                &mut old_protect,
            )
        };
        if res.as_bool() {
            Ok(old_protect)
        } else {
            match unsafe { GetLastError() } {
                e => {
                    println!("Error: {:?}", e);
                }
            }
            Err(ProcessError::UnableToChangeProtection(addr as usize).into())
        }
    }

    pub fn get_base_module(&'a self) -> Result<Module<'a>> {
        Module::new(&self.name, self)
    }
}
#[derive(Debug)]
pub enum StringOru32 {
    String(String),
    U32(u32),
}
impl Display for StringOru32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StringOru32::String(s) => write!(f, "{}", s),
            StringOru32::U32(u) => write!(f, "{}", u),
        }
    }
}

#[derive(Debug, Error)]
pub enum ProcessError {
    #[error("Unable open process with pid or name of '{0}'")]
    UnableToOpenProcess(StringOru32),
    #[error("Unable pid '{0}' does not exist")]
    InvalidPid(u32),
    #[error("unable to find any process with pid or name of '{0}'")]
    NoProcessFound(StringOru32),
    #[error("unable to read memory @ {0:X}")]
    UnableToReadMemory(usize),
    #[error("unable to write memory @ {0:X}")]
    UnableToWriteMemory(usize),
    #[error("unable to change memory protection @ {0:X}")]
    UnableToChangeProtection(usize),
}

impl Drop for Process {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.handl);
        }
    }
}
