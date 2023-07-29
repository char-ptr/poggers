
use crate::{traits::MemError, sigscan::SigScan};

use super::Module;

impl<T> Module<T> where T: SigScan {
    #[cfg(windows)]
    /// scan for a pattern in the module
    pub fn scan(&self,pattern:&str) -> Result<Option<usize>,MemError> {
        use windows::Win32::System::Memory::{MEM_COMMIT, PAGE_NOACCESS};
        let mut addr = self.get_base_address();
        
        while addr < self.get_base_address() + self.get_size() {
            unsafe {
                let query = self.get_owner().raw_query(addr);
                if query.State != MEM_COMMIT || query.Protect == PAGE_NOACCESS {
                    addr += query.RegionSize;
                    continue;
                }
                let mut page = [0u8; 0x4096];
                self.get_owner().raw_read(addr, &mut page as *mut u8, 0x4096)?;
                let scan_res = self.get_owner().scan(pattern, page.iter());
                
                if let Some(result) = scan_res {
                    println!("Found pattern at {:#x}", scan_res.unwrap());
                    return Ok(Some(addr + result));
                }
                addr += 0x4096_usize;
            }
        }
        Ok(None)
    }
    #[cfg(windows)]
    /// scan for a value of <V> in the module
    pub fn scan_value<V>(&self, val:&V) -> Result<Option<usize>,MemError> {
        use windows::Win32::System::Memory::{MEM_COMMIT, PAGE_NOACCESS};
        let mut addr = self.get_base_address();

        while addr < self.get_base_address() + self.get_size() {
            unsafe {
                let query = self.get_owner().raw_query(addr);
                if query.State != MEM_COMMIT || query.Protect == PAGE_NOACCESS {
                    addr += query.RegionSize;
                    continue;
                }
                let mut page = [0u8; 0x4096];
                self.get_owner().raw_read(addr, &mut page as *mut u8, 0x4096)?;
                let scan_res = self.get_owner().scan_batch_value(val, &page);
    
                if let Some(result) = scan_res {
                    println!("Found pattern at {:#x}", scan_res.unwrap());
                    return Ok(Some(addr + result));
                }
                addr += 0x4096_usize;
            }
        }
        Ok(None)
    }
}