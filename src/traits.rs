
use std::rc::Rc;

use thiserror::Error;

use crate::{structures::{virtalloc::VirtAlloc, addr::Address}, sigscan::SigScan};

use super::structures::protections::Protections;


/// trait which gives cross platform memory reading/writing etc.
pub trait Mem {
    /// Read <T> from memory at address <addr>
    /// This function is cross-platform.
    /// ```rs
    /// let data: u32 = process.read::<u32>(0x12345678)?;
    /// ```
    /// # Safety
    /// unsafe because it does direct calls to the OS. the address supplied could be invalid.
    unsafe fn read<T>(&self, addr: usize) -> Result<T,MemError> {
        let mut data: T = std::mem::zeroed();
        // if Self::READ_REQUIRE_PROTECTION {
        //     let old = self.alter_protection(addr, std::mem::size_of::<T>(), Protections::ExecuteReadWrite)?;
        //     self.raw_read(addr, &mut data as *mut T as *mut u8, std::mem::size_of::<T>())?;
        //     self.alter_protection(addr, std::mem::size_of::<T>(), old)?;
        // }
        // else {
            self.raw_read(addr, &mut data as *mut T as *mut u8, std::mem::size_of::<T>())?;
        // }
        // self.raw_read(addr, &mut data as *mut T as *mut u8, std::mem::size_of::<T>())?;
        Ok(data)
    }
    /// Read raw bytes from memory at address <addr> with size <size>
    /// # Safety
    /// unsafe because it does direct calls to the OS. the address supplied could be invalid.

    unsafe fn read_sized(&self, addr: usize,size:usize) -> Result<Vec<u8>,MemError> {
        let mut data: Vec<u8> = vec![0;size];
        // if Self::READ_REQUIRE_PROTECTION {
        //     let old = self.alter_protection(addr, size, Protections::ExecuteReadWrite)?;
        self.raw_read(addr, data.as_mut_ptr(), size)?;
        //     self.alter_protection(addr, size, old)?;
        // }
        Ok(data)
    }    
    /// Write <T> to memory at address <addr>
    /// # Safety
    /// unsafe because it does direct calls to the OS. the address supplied could be invalid.

    unsafe fn write<T>(&self, addr: usize, data: &T) -> Result<(),MemError> {
        // if Self::WRITE_REQUIRE_PROTECTION {
        //     let old = self.alter_protection(addr, std::mem::size_of::<T>(),Protections::ReadWrite)?;
        //     self.raw_write(addr, data as *const T as *const u8, std::mem::size_of::<T>())?;
        //     self.alter_protection(addr, std::mem::size_of::<T>(), old)?;
        // }
        // else {
            self.raw_write(addr, data as *const T as *const u8, std::mem::size_of::<T>())?;
        // }
        Ok(())
    }
    /// Write raw bytes to memory at address <addr>
    /// # Safety
    /// unsafe because it does direct calls to the OS. the address supplied could be invalid.

    unsafe fn write_raw(&self, addr: usize, data: &[u8]) -> Result<(),MemError> {
        
        // if Self::WRITE_REQUIRE_PROTECTION {
        //     let old = self.alter_protection(addr, data.len(),Protections::ReadWrite)?;
        //     self.raw_write(addr, data.as_ptr(), data.len())?;
        //     self.alter_protection(addr, data.len(), old)?;
        // }
        // else {
            self.raw_write(addr, data.as_ptr(), data.len())?;
        // }
        Ok(())
    }
    /// Fetch a page of memory at address <addr>
    /// # Safety
    /// unsafe because it does direct calls to the OS. the address supplied could be invalid.

    unsafe fn fetch_page(&self, addr: usize) -> Result<[u8; 0x1000],MemError> {
        let mut data: [u8; 0x1000] = [0; 0x1000];
        self.raw_read(addr, data.as_mut_ptr(), 0x1000)?;
        Ok(data)
    }
    /// get a wrapper around an address
    fn address(&self, size: usize) -> Address<Self> where Self : Sized + SigScan + Clone {
        Address::new(Rc::new(self.clone()), size)
    }
    /// Allocate memory to process begninning at <addr> with size <size>, needs implementation per platform
    /// This will automatically free the memory when the VirtAlloc is dropped
    /// To prevent this from happening use forget.
    /// # Safety
    /// unsafe because it does direct calls to the OS. the address supplied could be invalid.
    #[must_use = "keep the virtalloc alive to keep the memory allocated"]
    unsafe fn virtual_alloc(&self, addr: Option<usize>, size: usize, prot: Protections) -> Result<VirtAlloc<Self>,MemError> where Self: Sized {
        let addr = self.raw_virtual_alloc(addr, size, prot)?;
        Ok(VirtAlloc {
            addr,
            size,
            proc: self
        })
    }
    #[cfg(windows)]
    /// Query a page of memory at address <addr>
    /// # Safety
    /// unsafe because it does direct calls to the OS. the address supplied could be invalid.
    unsafe fn raw_query(&self, addr : usize) -> windows::Win32::System::Memory::MEMORY_BASIC_INFORMATION;
    /// Alter the protection of a memory region, needs implementation per platform
    /// # Safety
    /// unsafe because it does direct calls to the OS. the address supplied could be invalid.

    unsafe fn alter_protection(&self,addr:usize, size: usize, prot: Protections) -> Result<Protections,MemError>;
    /// Read raw bytes from memory at address <addr> with size <size>, needs implementation per platform
    /// # Safety
    /// unsafe because it does direct calls to the OS. the address supplied could be invalid.

    unsafe fn raw_read(&self, addr: usize,data: *mut u8, size: usize) -> Result<(),MemError>;
    /// Write raw bytes to memory at address <addr> with size <size>, needs implementation per platform
    /// # Safety
    /// unsafe because it does direct calls to the OS. the address supplied could be invalid.

    unsafe fn raw_write(&self, addr: usize,data: *const u8, size: usize) -> Result<(),MemError>;
    /// Allocate memory to process begninning at <addr> with size <size>, needs implementation per platform
    /// # Safety
    /// unsafe because it does direct calls to the OS. the address supplied could be invalid.

    unsafe fn raw_virtual_alloc(&self, addr:Option<usize>, size:usize, prot: Protections) -> Result<usize,MemError>;
    /// Free memory at process beginning at <addr> with size <size>, needs implementation per platform
    /// # Safety
    /// unsafe because it does direct calls to the OS. the address supplied could be invalid.
    unsafe fn raw_virtual_free(&self, addr:usize, size:usize) -> Result<(),MemError>;

}

/// Mem-trait Failures
#[derive(Debug,Error)]
pub enum MemError {
    /// Read failed
    #[error("Read failed [{0:X}]")]
    ReadFailure(usize),
    /// Write failed
    #[error("Write failed [{0:X}]")]
    WriteFailure(usize),
    /// Protection update failed
    #[error("Protection update to {1} failed [{0:X}]+{1:X}")]
    ProtectFailure(usize,usize, Protections),
    /// Unable to allocate memory
    #[error("VirtualAlloc failed [{0:X?}]+{1:X}")]
    AllocFailure(Option<usize>, usize),
    /// Failed to free memory
    #[error("VirtualFree failed [{0:X}]+{1:X}")]
    FreeFailure(usize, usize),
}