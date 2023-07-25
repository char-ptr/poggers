use anyhow::Result;
use std::{ffi::c_void, fmt::Display};
use thiserror::Error;

use windows::Win32::{
    Foundation::{CloseHandle, GetLastError, HANDLE, MAX_PATH},
    System::{
        Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory},
        Memory::{VirtualAllocEx, VirtualProtectEx, MEM_COMMIT, MEM_RESERVE},
        ProcessStatus::GetModuleFileNameExW,
        Threading::{GetExitCodeProcess, OpenProcess, TerminateProcess, PROCESS_ALL_ACCESS},
    },
};

use crate::structures::{Protections, VirtAlloc};
use crate::traits::Mem;

use super::{
    create_snapshot::{STProcess, ToolSnapshot},
    module::ExModule,
};

/// A Class describing a windows process
#[derive(Debug)]
pub struct ExProcess {
    pub(crate) handl: windows::Win32::Foundation::HANDLE,
    pub(crate) pid: u32,
    pub(crate) name: String,
}

impl<'a> ExProcess {
    /// returns the current process id for the process
    pub fn get_pid(&self) -> u32 {
        self.pid
    }

    /// Creates a new [`ExProcess`] from a pid
    /// # Example
    /// ```
    /// use poggers::external::process::ExProcess;
    /// let proc = ExProcess::new(1234).unwrap();
    /// ```
    pub fn new_from_pid(pid: u32) -> Result<ExProcess> {
        let handle = Self::open_handle(pid)?;
        let process_name = Self::get_name_from_pid(pid, &handle)?;

        Ok(ExProcess {
            handl: handle,
            pid,
            name: process_name,
        })
    }
    /// create a new [`ExProcess`] from a process name
    /// # Example
    /// ```
    /// use poggers::external::process::ExProcess;
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
    /// # Example
    /// ```
    /// use poggers::external::process::ExProcess;
    /// let mut process = ExProcess::new_from_name("notepad.exe".to_string()).unwrap();
    /// assert!(process.alive());
    /// ```
    pub fn alive(&self) -> bool {
        let mut exit_code: u32 = 0;
        unsafe { GetExitCodeProcess(self.handl, &mut exit_code as *mut u32) }.as_bool()
            && exit_code == 259
    }

    fn get_pid_from_name(proc_name: &str) -> Result<u32> {
        let mut snapshot = ToolSnapshot::new_process()?;
        snapshot
            .find(|x| x.exe_path == proc_name)
            .map(|x| x.id)
            .ok_or(ProcessError::NoProcessFound(StringOru32::String(proc_name.to_string())).into())
    }
    /// get the name of the process
    pub fn get_name_from_pid(process_id: u32, hndl: &HANDLE) -> Result<String> {
        if process_id == 0 {
            return Err(ProcessError::InvalidPid(process_id).into());
        }
        let mut buffer = [0u16; MAX_PATH as usize];
        unsafe { GetModuleFileNameExW(*hndl, None, &mut buffer) };
        // println!("{:?}", buffer);
        Ok(String::from_utf16_lossy(&buffer)
            .rsplit('\\')
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
    /// Get the base module of the process (name.exe module)
    /// # Return
    /// * [Result]<[ExModule]> - The base module of the process
    /// # Example
    /// ```
    /// use poggers::external::process::ExProcess;
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

    /// kill the process, will always exit with code 0
    pub fn kill(self) -> bool {
        unsafe { TerminateProcess(self.handl, 0).as_bool() }
    }
}

impl Mem for ExProcess {
    unsafe fn alter_protection(
        &self,
        addr: usize,
        size: usize,
        prot: Protections,
    ) -> Result<Protections> {
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
            // plan to match in the future, cba atm
            match unsafe { GetLastError() } {
                e => {
                    println!("Error: {:?}", e);
                }
            }
            Err(ProcessError::UnableToChangeProtection(addr).into())
        }
    }
    unsafe fn raw_read(&self, addr: usize, data: *mut u8, size: usize) -> Result<()> {
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
            Err(ProcessError::UnableToReadMemory(addr).into())
        }
    }
    unsafe fn raw_write(&self, addr: usize, data: *const u8, size: usize) -> Result<()> {
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
            Err(ProcessError::UnableToWriteMemory(addr).into())
        }
    }
    unsafe fn virtual_alloc(
        &self,
        addr: usize,
        size: usize,
        prot: Protections,
    ) -> Result<crate::structures::VirtAlloc> {
        let alloc_ret = VirtualAllocEx(
            self.handl,
            Some(addr as *mut c_void),
            size,
            MEM_COMMIT | MEM_RESERVE,
            prot.native(),
        );
        if alloc_ret.is_null() {
            Err(ProcessError::UnableToAllocate(size, addr).into())
        } else {
            Ok(VirtAlloc {
                pid: self.pid,
                addr,
                size,
            })
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
    #[error("unable open process with pid or name of '{0}'")]
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
    /// unable to allocate memory
    #[error("unable to allocate memory of size {0} to {1:X}")]
    UnableToAllocate(usize, usize),
}

impl Drop for ExProcess {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.handl);
        }
    }
}
impl From<STProcess> for ExProcess {
    fn from(val: STProcess) -> Self {
        ExProcess::new_from_pid(val.id).unwrap()
    }
}
