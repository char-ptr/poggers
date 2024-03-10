use std::{ffi::CStr, marker::PhantomData};

use thiserror::Error;
use windows::Win32::{
    Foundation::{CloseHandle, HANDLE, HMODULE},
    System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Module32First, Module32Next, Process32First, Process32Next,
        MODULEENTRY32, PROCESSENTRY32, TH32CS_SNAPMODULE, TH32CS_SNAPMODULE32, TH32CS_SNAPPROCESS,
    },
};

/// Represents a process from ToolSnapshotHelper
#[derive(Debug, Clone)]
pub struct STProcess {
    /// the process id
    pub id: u32,
    /// the amount of threads in the process
    pub thread_count: u32,
    /// the parent process id
    pub parent_id: u32,
    /// the name to the executable
    pub exe_path: String,
}
impl PartialEq for STProcess {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

// struct STThread;
/// Represents a module from ToolSnapshotHelper
pub struct STModule {
    /// the process id
    pub process_id: u32,
    /// the base address of the module
    pub base_address: usize,
    /// the size of the module
    pub size: usize,
    /// the name of the module
    pub name: String,
    /// the file name of the module
    pub exe_path: String,
    /// a handle to the module <DO NOT DISPOSE OF>
    pub handle: HMODULE,
}
/// used to provide which type of snapshot to create
pub struct NoTypeSel;
/// represents a snapshot of the processes or modules
pub struct ToolSnapshot<T = NoTypeSel> {
    handle: HANDLE,
    first_complete: bool,
    phantom: PhantomData<T>,
}
impl ToolSnapshot {
    /// Create a new process snapshot
    #[inline(always)]
    pub fn new_process() -> Result<ToolSnapshot<STProcess>, SnapshotToolError> {
        ToolSnapshot::<STProcess>::new()
    }
    /// Create a new module snapshot
    #[inline(always)]
    pub fn new_module(proc_id: Option<u32>) -> Result<ToolSnapshot<STModule>, SnapshotToolError> {
        ToolSnapshot::<STModule>::new(proc_id)
    }
}
impl ToolSnapshot<STModule> {
    /// Creates a new module snapshot
    pub fn new(proc_id: Option<u32>) -> Result<Self, SnapshotToolError> {
        let handle = unsafe {
            CreateToolhelp32Snapshot(
                TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32,
                proc_id.unwrap_or(0),
            )
        };
        handle
            .map(|handle| Self {
                handle,
                first_complete: false,
                phantom: PhantomData,
            })
            .or(Err(SnapshotToolError::CreateSnapshotError))
    }
}
impl Iterator for ToolSnapshot<STModule> {
    type Item = STModule;
    fn next(&mut self) -> Option<Self::Item> {
        let mut lpme = MODULEENTRY32 {
            dwSize: std::mem::size_of::<MODULEENTRY32>() as u32,
            ..Default::default()
        };
        return if !self.first_complete {
            unsafe {
                if Module32First(self.handle, &mut lpme).is_ok() {
                    self.first_complete = true;
                    let proc_name =
                        std::slice::from_raw_parts(lpme.szModule.as_ptr() as *const u8, 256);
                    let proc_path =
                        std::slice::from_raw_parts(lpme.szExePath.as_ptr() as *const u8, 260);
                    Some(STModule {
                        process_id: lpme.th32ProcessID,
                        base_address: lpme.modBaseAddr as usize,
                        size: lpme.modBaseSize as usize,
                        name: CStr::from_bytes_until_nul(proc_name)
                            .unwrap()
                            .to_string_lossy()
                            .to_string(),
                        exe_path: CStr::from_bytes_until_nul(proc_path)
                            .unwrap()
                            .to_string_lossy()
                            .to_string(),
                        handle: lpme.hModule,
                    })
                } else {
                    None
                }
            }
        } else {
            unsafe {
                if Module32Next(self.handle, &mut lpme).is_ok() {
                    let proc_name =
                        std::slice::from_raw_parts(lpme.szModule.as_ptr() as *const u8, 256);
                    let proc_path =
                        std::slice::from_raw_parts(lpme.szExePath.as_ptr() as *const u8, 260);
                    Some(STModule {
                        process_id: lpme.th32ProcessID,
                        base_address: lpme.modBaseAddr as usize,
                        size: lpme.modBaseSize as usize,
                        name: CStr::from_bytes_until_nul(proc_name)
                            .unwrap()
                            .to_string_lossy()
                            .to_string(),
                        exe_path: CStr::from_bytes_until_nul(proc_path)
                            .unwrap()
                            .to_string_lossy()
                            .to_string(),
                        handle: lpme.hModule,
                    })
                } else {
                    None
                }
            }
        };
    }
}
impl ToolSnapshot<STProcess> {
    /// Creates a new process snapshot
    pub fn new() -> Result<Self, SnapshotToolError> {
        let handle = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
        handle
            .map(|handle| Self {
                handle,
                first_complete: false,
                phantom: PhantomData,
            })
            .or(Err(SnapshotToolError::CreateSnapshotError))
    }
}
impl Iterator for ToolSnapshot<STProcess> {
    type Item = STProcess;
    fn next(&mut self) -> Option<Self::Item> {
        let mut lppe = PROCESSENTRY32 {
            dwSize: std::mem::size_of::<MODULEENTRY32>() as u32,
            ..Default::default()
        };
        return if !self.first_complete {
            unsafe {
                if Process32First(self.handle, &mut lppe).is_ok() {
                    self.first_complete = true;
                    let mod_path =
                        std::slice::from_raw_parts(lppe.szExeFile.as_ptr() as *const u8, 260);
                    Some(STProcess {
                        id: lppe.th32ProcessID,
                        thread_count: lppe.cntThreads,
                        parent_id: lppe.th32ParentProcessID,
                        exe_path: CStr::from_bytes_until_nul(mod_path)
                            .unwrap()
                            .to_string_lossy()
                            .to_string(),
                    })
                } else {
                    None
                }
            }
        } else {
            unsafe {
                if Process32Next(self.handle, &mut lppe).is_ok() {
                    let mod_path =
                        std::slice::from_raw_parts(lppe.szExeFile.as_ptr() as *const u8, 260);
                    Some(STProcess {
                        id: lppe.th32ProcessID,
                        thread_count: lppe.cntThreads,
                        parent_id: lppe.th32ParentProcessID,
                        exe_path: CStr::from_bytes_until_nul(mod_path)
                            .unwrap()
                            .to_string_lossy()
                            .to_string(),
                    })
                } else {
                    None
                }
            }
        };
    }
}
impl<T> Drop for ToolSnapshot<T> {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.handle).ok();
        }
    }
}
/// Errors that can occur when creating a snapshot
#[derive(Error, Debug)]
pub enum SnapshotToolError {
    /// Failed to create snapshot
    #[error("Failed to create snapshot")]
    CreateSnapshotError,
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_process() {
        let mut snapshot = ToolSnapshot::new_process().unwrap();
        assert!(snapshot.any(|x| x.exe_path.contains("poggers")))
    }
    #[test]
    fn test_module() {
        let mut snapshot = ToolSnapshot::new_module(None).unwrap();
        assert!(snapshot.any(|x| x.name == "ntdll.dll"))
    }
}
