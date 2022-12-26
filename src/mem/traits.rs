use anyhow::Result;

use super::structures::Protections;


/// trait which gives cross platform memory reading/writing etc.
pub trait Mem {
    /// If the platform requires protection to be changed before writing.
    const WRITE_REQUIRE_PROTECTION: bool = false;
    /// If the platform requires protection to be changed before reading.
    const READ_REQUIRE_PROTECTION: bool = false;
    /// The size of a page.
    const PAGE_SIZE: usize = 0x1000;
    /// Read <T> from memory at address <addr>
    /// This function is crossplatform.
    /// ```rs
    /// let data: u32 = process.read::<u32>(0x12345678)?;
    /// ```
    unsafe fn read<T>(&self, addr: usize) -> Result<T> {
        let mut data: T = std::mem::zeroed();
        if Self::READ_REQUIRE_PROTECTION {
            let old = self.alter_protection(addr, std::mem::size_of::<T>(), Protections::ExecuteReadWrite)?;
            self.raw_read(addr, &mut data as *mut T as *mut u8, std::mem::size_of::<T>())?;
            self.alter_protection(addr, std::mem::size_of::<T>(), old)?;
        }
        else {
            self.raw_read(addr, &mut data as *mut T as *mut u8, std::mem::size_of::<T>())?;
        }
        // self.raw_read(addr, &mut data as *mut T as *mut u8, std::mem::size_of::<T>())?;
        Ok(data)
    }
    /// Read raw bytes from memory at address <addr> with size <size>
    unsafe fn read_sized(&self, addr: usize,size:usize) -> Result<Vec<u8>> {
        let mut data: Vec<u8> = vec![0;size];
        if Self::READ_REQUIRE_PROTECTION {
            let old = self.alter_protection(addr, size, Protections::ExecuteReadWrite)?;
            self.raw_read(addr, data.as_mut_ptr(), size)?;
            self.alter_protection(addr, size, old)?;
        }
        Ok(data)
    }
    /// Write <T> to memory at address <addr>
    unsafe fn write<T>(&self, addr: usize, data: &T) -> Result<()> {
        if Self::WRITE_REQUIRE_PROTECTION {
            let old = self.alter_protection(addr, std::mem::size_of::<T>(),Protections::ReadWrite)?;
            self.raw_write(addr, data as *const T as *const u8, std::mem::size_of::<T>())?;
            self.alter_protection(addr, std::mem::size_of::<T>(), old)?;
        }
        else {
            self.raw_write(addr, data as *const T as *const u8, std::mem::size_of::<T>())?;
        }
        Ok(())
    }
    /// Write raw bytes to memory at address <addr>
    unsafe fn write_raw(&self, addr: usize, data: &[u8]) -> Result<()> {
        
        if Self::WRITE_REQUIRE_PROTECTION {
            let old = self.alter_protection(addr, data.len(),Protections::ReadWrite)?;
            self.raw_write(addr, data.as_ptr(), data.len())?;
            self.alter_protection(addr, data.len(), old)?;
        }
        else {
            self.raw_write(addr, data.as_ptr(), data.len())?;
        }
        Ok(())
    }
    /// Fetch a page of memory at address <addr>
    unsafe fn fetch_page(&self, addr: usize) -> Result<[u8; Self::PAGE_SIZE]> {
        let mut data: [u8; Self::PAGE_SIZE] = [0; Self::PAGE_SIZE];
        self.raw_read(addr, data.as_mut_ptr(), Self::PAGE_SIZE)?;
        Ok(data)
    }

    /// Alter the protection of a memory region, needs implementation per platform
    unsafe fn alter_protection(&self,addr:usize, size: usize, prot: Protections) -> Result<Protections>;
    /// Read raw bytes from memory at address <addr> with size <size>, needs implementation per platform
    unsafe fn raw_read(&self, addr: usize,data: *mut u8, size: usize) -> Result<()>;
    /// Write raw bytes to memory at address <addr> with size <size>, needs implementation per platform
    unsafe fn raw_write(&self, addr: usize,data: *const u8, size: usize) -> Result<()>;

}