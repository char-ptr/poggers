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
        Threading::{OpenProcess, PROCESS_ALL_ACCESS, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ, GetExitCodeProcess},
    },
};

use crate::mem::traits::Mem;

use super::module::ExModule;


/// A Class describing a windows process
#[derive(Debug)]
pub struct ExProcess {
    pub(crate) handl: windows::Win32::Foundation::HANDLE,
    pub(crate) pid: u32,
    pub(crate) name: String,
}

impl<'a> ExProcess {
    /// Creates a new [`ExProcess`] from a pid
    /// # Example
    /// ```
    /// use mem::process::ExProcess;
    /// let proc = ExProcess::new(1234).unwrap();
    /// ```
    pub fn new_from_pid(pid: u32) -> Result<ExProcess> {
        let process_name = Self::get_name_from_pid(pid)?;
        let handle = Self::open_handle(pid)?;

        Ok(ExProcess {
            handl: handle,
            pid,
            name: process_name,
        })
    }
    /// create a new [`ExProcess`] from a process name
    /// # Example
    /// ```
    /// use mem::process::ExProcess;
    /// let process = ExProcess::new("csgo.exe").unwrap();
    /// ```
    pub fn new_from_name(name: String) -> Result<ExProcess> {
        let pid = Self::get_pid_from_name(&name)?;
        let handle = Self::open_handle(pid)?;

        Ok(ExProcess {
            handl: handle,
            pid,
            name,
        })
    }
    /// get the handle to the process
    pub fn get_handle(&self) -> &HANDLE {
        &self.handl
    }
    /// check that the process is still running
    pub fn alive(&self) -> bool {
        let mut exit_code: u32 = 0;
        unsafe {
            GetExitCodeProcess(
                self.handl,
                &mut exit_code as *mut u32,
            )
        }
        .as_bool()
            && exit_code == 259
    }

    /// return a [`Vec`] of [`ExPartialProcess`]s that are currently running
    pub fn get_processes() -> Result<Vec<ExPartialProcess>> {
        let mut processes = Vec::new();
        let snapshot = unsafe {
            CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)
                .ok()
                .context("Failed to create snapshot")?
        };
        let mut entry = PROCESSENTRY32::default();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;
        unsafe {Process32First(snapshot, &mut entry)};
        {
            let f = entry.szExeFile.iter().take_while(|x| x.0 != 0).map(|x| x.0).collect::<Vec<u8>>();
            processes.push(ExPartialProcess {
                pid: entry.th32ProcessID,
                name: String::from_utf8_lossy(&f).to_string(),
            });
            
        }
        while unsafe { Process32Next(snapshot, &mut entry) }.as_bool() {
            let f = entry.szExeFile.iter().take_while(|x| x.0 != 0).map(|x| x.0).collect::<Vec<u8>>();
            processes.push(ExPartialProcess {
                pid: entry.th32ProcessID,
                name: String::from_utf8_lossy(&f).to_string(),
            });
        }
        unsafe {
            CloseHandle(snapshot);
        }
        Ok(processes)
    }

    fn get_pid_from_name(proc_name: &str) -> Result<u32> {
        let mut pe: PROCESSENTRY32 = Default::default();
        pe.dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;

        let hsnapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }.or(Err(
            ProcessError::UnableToOpenProcess(StringOru32::String(proc_name.to_string())),
        )).context("unable to create tool help 32 snapshot")?;

        let mut result: Option<u32> = None;
        let le_poggier = |p: &PROCESSENTRY32| {
            let f = p.szExeFile.iter().take_while(|x| x.0 != 0).map(|x| x.0).collect::<Vec<u8>>();
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
    /// get the name of the process
    pub fn get_name_from_pid(process_id: u32) -> Result<String> {
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
    /// Resolve a vector of pointers to a single address
    /// # arguments
    /// * addr - the base address
    /// * offsets - a vector of offsets
    /// # example
    /// ```
    /// let mut process = ExProcess::new_from_name("notepad.exe".to_string()).unwrap();
    /// let addr = process.solve_dma(0x12345678, vec![0x0, 0x4, 0x8]).unwrap();
    /// ```
    pub unsafe fn solve_dma(&self, addr: usize, offsets: &Vec<usize>) -> Result<usize> {
        let mut ptr = addr;
        for offset in offsets {
            ptr = self.read::<usize>(addr)?;
            ptr += offset;
        }
        Ok(ptr)
    }
    /// Get the base module of the process (name.exe module)
    /// # Return
    /// * [Result]<[ExModule]> - The base module of the process
    /// # Example
    /// ```
    /// use poggers::process::ExProcess;
    /// let mut process = ExProcess::new_from_name("notepad.exe".to_string()).unwrap();
    /// let base_module = process.get_base_module().unwrap();
    /// ```
    pub fn get_base_module(&'a self) -> Result<ExModule<'a>> {
        ExModule::new(&self.name, self)
    }
    /// Get a module by name
    pub fn get_module(&'a self, name: &str) -> Result<ExModule<'a>> {
        ExModule::new(name, self)
    }


}

impl Mem for ExProcess {
    unsafe fn raw_read(&self, addr: usize,data: *mut u8, size: usize) -> Result<()> {
        let res = ReadProcessMemory(
            self.handl,
            addr as *const c_void,
            data as *mut _,
            size,
            Some(&mut 0),
        );


        if res.as_bool() {
            Ok(())
        } else {
            Err(ProcessError::UnableToReadMemory(addr as usize).into())
        }
    }
    unsafe fn raw_write(&self, addr: usize,data: *const u8, size: usize) -> Result<()> {
        let res = WriteProcessMemory(
            self.handl,
            addr as *const c_void,
            data as *const _,
            size,
            Some(&mut 0),
        );
        if res.as_bool() {
            Ok(())
        } else {
            Err(ProcessError::UnableToWriteMemory(addr as usize).into())
        }
    }
    unsafe fn alter_protection(&self,addr:usize, size: usize, prot: crate::mem::structures::Protections) -> Result<crate::mem::structures::Protections> {
        let mut old_protect = Default::default();
        let res = unsafe {
            VirtualProtectEx(
                self.handl,
                addr as *const c_void,
                size,
                prot.native(),
                &mut old_protect,
            )
        };
        if res.as_bool() {
            Ok(old_protect.0.into())
        } else {
            match unsafe { GetLastError() } {
                e => {
                    println!("Error: {:?}", e);
                }
            }
            Err(ProcessError::UnableToChangeProtection(addr as usize).into())
        }

    }
}

/// an enum which can be either a string or a u32
#[derive(Debug)]
pub enum StringOru32 {
    /// a string
    String(String),
    /// a u32
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
/// An error that can occur when interacting with a process
#[derive(Debug, Error)]
pub enum ProcessError {
    /// unable to open a handle to the process
    #[error("Unable open process with pid or name of '{0}'")]
    UnableToOpenProcess(StringOru32),
    /// the pid specified is invalid
    #[error("pid '{0}' does not exist")]
    InvalidPid(u32),
    /// process cannot be found with name
    #[error("unable to find any process with pid or name of '{0}'")]
    NoProcessFound(StringOru32),
    /// unable to read memory from the process
    #[error("unable to read memory @ {0:X}")]
    UnableToReadMemory(usize),
    /// unable to write memory to the process
    #[error("unable to write memory @ {0:X}")]
    UnableToWriteMemory(usize),
    /// unable to change the protection of the memory
    #[error("unable to change memory protection @ {0:X}")]
    UnableToChangeProtection(usize),
}

impl Drop for ExProcess {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.handl);
        }
    }
}
/// A Smaller process, which doesn't contain a handle to the process
#[derive(Debug,Clone,Default)]
pub struct ExPartialProcess {
    /// the pid of the process
    pub pid: u32,
    /// the name of the process
    pub name: String,
}
impl ExPartialProcess {
    /// Create a new ExPartialProcess
    fn new(pid: u32, name: String) -> Self {
        Self { pid, name }
    }
    /// convert the ExPartialProcess into an ExProcess
    fn full(self) -> Result<ExProcess> {
        ExProcess::new_from_pid(self.pid)
    }
}
impl std::cmp::PartialEq for ExPartialProcess {
    fn eq(&self, other: &Self) -> bool {
        self.pid == other.pid
    }
}