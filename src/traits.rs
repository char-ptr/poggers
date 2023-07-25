
use thiserror::Error;

use crate::structures::VirtAlloc;

use super::structures::Protections;


/// trait which gives cross platform memory reading/writing etc.
pub trait Mem {
    /// Read <T> from memory at address <addr>
    /// This function is cross-platform.
    /// ```rs
    /// let data: u32 = process.read::<u32>(0x12345678)?;
    /// ```
    /// # Safety
    /// this should never panic even if you provide invalid addresses
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
    /// this should never panic even if you provide invalid addresses

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
    /// this should never panic even if you provide invalid addresses

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
    /// this should never panic even if you provide invalid addresses

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
    /// this should never panic even if you provide invalid addresses

    unsafe fn fetch_page(&self, addr: usize) -> Result<[u8; 0x1000],MemError> {
        let mut data: [u8; 0x1000] = [0; 0x1000];
        self.raw_read(addr, data.as_mut_ptr(), 0x1000)?;
        Ok(data)
    }

    /// Alter the protection of a memory region, needs implementation per platform
    /// # Safety
    /// this should never panic even if you provide invalid addresses

    unsafe fn alter_protection(&self,addr:usize, size: usize, prot: Protections) -> Result<Protections,MemError>;
    /// Read raw bytes from memory at address <addr> with size <size>, needs implementation per platform
    /// # Safety
    /// this should never panic even if you provide invalid addresses

    unsafe fn raw_read(&self, addr: usize,data: *mut u8, size: usize) -> Result<(),MemError>;
    /// Write raw bytes to memory at address <addr> with size <size>, needs implementation per platform
    /// # Safety
    /// this should never panic even if you provide invalid addresses

    unsafe fn raw_write(&self, addr: usize,data: *const u8, size: usize) -> Result<(),MemError>;
    /// Allocate memory to process begninning at <addr> with size <size>, needs implementation per platform
    /// # Safety
    /// this should never panic even if you provide invalid addresses

    unsafe fn virtual_alloc(&self, addr: usize, size: usize, prot: Protections) -> Result<VirtAlloc,MemError>;

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
    #[error("VirtualAlloc failed [{0:X}]+{1:X}")]
    AllocFailure(usize, usize),
}