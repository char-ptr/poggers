use anyhow::Result;

use super::structures::Protections;



pub trait Mem {
    const WRITE_REQUIRE_PROTECTION: bool = false;
    const READ_REQUIRE_PROTECTION: bool = false;
    const PAGE_SIZE: usize = 0x1000;
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
    unsafe fn read_sized(&self, addr: usize,size:usize) -> Result<Vec<u8>> {
        let mut data: Vec<u8> = vec![0;size];
        if Self::READ_REQUIRE_PROTECTION {
            let old = self.alter_protection(addr, size, Protections::ExecuteReadWrite)?;
            self.raw_read(addr, data.as_mut_ptr(), size)?;
            self.alter_protection(addr, size, old)?;
        }
        Ok(data)
    }
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
    unsafe fn fetch_page(&self, addr: usize) -> Result<[u8; Self::PAGE_SIZE]> {
        let mut data: [u8; Self::PAGE_SIZE] = [0; Self::PAGE_SIZE];
        self.raw_read(addr, data.as_mut_ptr(), Self::PAGE_SIZE)?;
        Ok(data)
    }
    unsafe fn alter_protection(&self,addr:usize, size: usize, prot: Protections) -> Result<Protections>;
    
    unsafe fn raw_read(&self, addr: usize,data: *mut u8, size: usize) -> Result<()>;
    unsafe fn raw_write(&self, addr: usize,data: *const u8, size: usize) -> Result<()>;

}