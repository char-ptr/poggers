use anyhow::Result;

use crate::structures::VirtAlloc;

use super::structures::Protections;

/// trait which gives cross platform memory reading/writing etc.
pub trait Mem {
    /// The size of a page.
    const PAGE_SIZE: usize = 0x1000;
    /// Read <T> from memory at address <addr>
    /// This function is cross-platform.
    /// ```rs
    /// let data: u32 = process.read::<u32>(0x12345678)?;
    /// ```
    /// # Safety
    /// this should never panic even if you provide invalid addresses
    unsafe fn read<T>(&self, addr: usize) -> Result<T> {
        let mut data: T = std::mem::zeroed();
        // if Self::READ_REQUIRE_PROTECTION {
        //     let old = self.alter_protection(addr, std::mem::size_of::<T>(), Protections::ExecuteReadWrite)?;
        //     self.raw_read(addr, &mut data as *mut T as *mut u8, std::mem::size_of::<T>())?;
        //     self.alter_protection(addr, std::mem::size_of::<T>(), old)?;
        // }
        // else {
        self.raw_read(
            addr,
            &mut data as *mut T as *mut u8,
            std::mem::size_of::<T>(),
        )?;
        // }
        // self.raw_read(addr, &mut data as *mut T as *mut u8, std::mem::size_of::<T>())?;
        Ok(data)
    }
    /// Read raw bytes from memory at address <addr> with size <size>
    /// # Safety
    /// this should never panic even if you provide invalid addresses

    unsafe fn read_sized(&self, addr: usize, size: usize) -> Result<Vec<u8>> {
        let mut data: Vec<u8> = vec![0; size];
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

    unsafe fn write<T>(&self, addr: usize, data: &T) -> Result<()> {
        // if Self::WRITE_REQUIRE_PROTECTION {
        //     let old = self.alter_protection(addr, std::mem::size_of::<T>(),Protections::ReadWrite)?;
        //     self.raw_write(addr, data as *const T as *const u8, std::mem::size_of::<T>())?;
        //     self.alter_protection(addr, std::mem::size_of::<T>(), old)?;
        // }
        // else {
        self.raw_write(
            addr,
            data as *const T as *const u8,
            std::mem::size_of::<T>(),
        )?;
        // }
        Ok(())
    }
    /// Write raw bytes to memory at address <addr>
    /// # Safety
    /// this should never panic even if you provide invalid addresses

    unsafe fn write_raw(&self, addr: usize, data: &[u8]) -> Result<()> {
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

    unsafe fn fetch_page(&self, addr: usize) -> Result<[u8; Self::PAGE_SIZE]> {
        let mut data: [u8; Self::PAGE_SIZE] = [0; Self::PAGE_SIZE];
        self.raw_read(addr, data.as_mut_ptr(), Self::PAGE_SIZE)?;
        Ok(data)
    }

    /// Resolve a vector of pointers to a single address
    /// # arguments
    /// * addr - the base address
    /// * offsets - a vector of offsets
    /// # example
    /// ```
    /// use poggers::external::process::ExProcess;
    /// let mut process = ExProcess::new_from_name("notepad.exe".to_string()).unwrap();
    /// unsafe {
    ///     let addrs: Vec<usize> = vec![0x0, 0x4, 0x8];
    ///     let addr = process.solve_dma(0x12345678, &addrs).unwrap();
    /// }
    /// ```
    /// # Safety
    /// safe to use if address is valid and offsets don't go down into a null pointer
    
    unsafe fn solve_dma(&self, addr: usize, offsets: &Vec<usize>) -> Result<usize> {
        let mut ptr = addr;
        for offset in offsets {
            ptr = self.read::<usize>(addr)?;
            ptr += offset;
        }
        Ok(ptr)
    }

    /// Alter the protection of a memory region, needs implementation per platform
    /// # Safety
    /// this should never panic even if you provide invalid addresses

    unsafe fn alter_protection(
        &self,
        addr: usize,
        size: usize,
        prot: Protections,
    ) -> Result<Protections>;
    /// Read raw bytes from memory at address <addr> with size <size>, needs implementation per platform
    /// # Safety
    /// this should never panic even if you provide invalid addresses

    unsafe fn raw_read(&self, addr: usize, data: *mut u8, size: usize) -> Result<()>;
    /// Write raw bytes to memory at address <addr> with size <size>, needs implementation per platform
    /// # Safety
    /// this should never panic even if you provide invalid addresses

    unsafe fn raw_write(&self, addr: usize, data: *const u8, size: usize) -> Result<()>;
    /// Allocate memory to process begninning at <addr> with size <size>, needs implementation per platform
    /// # Safety
    /// this should never panic even if you provide invalid addresses

    unsafe fn virtual_alloc(
        &self,
        addr: usize,
        size: usize,
        prot: Protections,
    ) -> Result<VirtAlloc>;
}
