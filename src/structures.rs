/// Memory Protection Flags
pub enum Protections {
    /// If memory can execute in this page ?
    Execute,
    /// If memory can be read and executed
    ExecuteRead,
    /// if memory can be read, write, and executed
    ExecuteReadWrite,
    /// win32 specific
    ExecuteWriteCopy,
    /// No Permissions
    NoAccess,
    /// only read access
    ReadOnly,
    /// can read or write
    ReadWrite,
    /// win32 specific
    WriteCopy,
    /// win32 specific
    TargetInvalid,
    /// win32 specific
    TargerNoUpdate,
    /// invalid protection
    INVALID,
}
use std::ffi::c_void;

#[cfg(windows)]
use windows::Win32::System::Memory::PAGE_PROTECTION_FLAGS;
use windows::Win32::System::Memory::{VirtualFreeEx, MEM_RELEASE};

use crate::external::process::ExProcess;
#[cfg(windows)]
impl Protections {
    /// convert into u32
    pub fn u32(&self) -> u32 {
        match &self {
            Protections::Execute => 0x10,
            Protections::ExecuteRead => 0x20,
            Protections::ExecuteReadWrite => 0x40,
            Protections::ExecuteWriteCopy => 0x80,
            Protections::NoAccess => 0x01,
            Protections::ReadOnly => 0x02,
            Protections::ReadWrite => 0x04,
            Protections::WriteCopy => 0x08,
            Protections::TargetInvalid => 0x40000000,
            Protections::TargerNoUpdate => 0x40000000,
            Protections::INVALID => 0x0,
        }
    }
    /// convert into native version 
    pub fn native(&self) -> PAGE_PROTECTION_FLAGS {
        PAGE_PROTECTION_FLAGS(self.u32())
    }
}
#[cfg(windows)]
impl From<u32> for Protections {
    fn from(value: u32) -> Self {
        match value {
            0x10 => Protections::Execute,
            0x20 => Protections::ExecuteRead,
            0x40 => Protections::ExecuteReadWrite,
            0x80 => Protections::ExecuteWriteCopy,
            0x01 => Protections::NoAccess,
            0x02 => Protections::ReadOnly,
            0x04 => Protections::ReadWrite,
            0x08 => Protections::WriteCopy,
            0x40000000 => Protections::TargetInvalid,
            _ => Protections::INVALID,
        }
    }
}
#[cfg(windows)]
impl From<Protections> for u32 {
    fn from(val: Protections) -> Self {
        val.u32()
    }
}

/// Allocted memory
pub struct VirtAlloc {
    pub(crate) pid: u32,
    pub(crate) addr: usize,
    pub(crate) size: usize
}

impl VirtAlloc {
    #[cfg(windows)]
    /// Free the allocated memory
    pub fn free(self) {
        let Ok(proc) = ExProcess::new_from_pid(self.pid) else {
            return
        };

        unsafe {
            VirtualFreeEx(proc.handl, self.addr as *mut c_void, self.size, MEM_RELEASE);
        }
    }

    /// Get address of allocated memory
    pub fn get_addr(&self) -> usize {
        self.addr
    }

    /// Get size of allocated memory
    pub fn get_size(&self) -> usize {
        self.size
    }
}

impl Drop for VirtAlloc {
    #[cfg(windows)]
    fn drop(&mut self) {
        let Ok(proc) = ExProcess::new_from_pid(self.pid) else {
            return
        };

        unsafe {
            VirtualFreeEx(proc.handl, self.addr as *mut c_void, self.size, MEM_RELEASE);
        }
    }
}
