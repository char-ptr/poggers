use crate::{sigscan::SigScan, traits::MemError};

use super::Module;

impl<T> Module<T>
where
    T: SigScan,
{
    #[cfg(windows)]
    /// scan for a pattern in the module
    pub fn scan(&self, pattern: &str) -> Result<Option<usize>, MemError> {
        use windows::Win32::System::Memory::{MEM_COMMIT, PAGE_NOACCESS};
        let mut addr = self.get_base_address();
        let owner = self.get_owner();
        // this should have no performance implications cause get_base_address.. is const
        while addr < self.get_base_address() + self.get_size() {
            unsafe {
                let query = owner.raw_query(addr);
                if query.State != MEM_COMMIT || query.Protect == PAGE_NOACCESS {
                    addr += query.RegionSize;
                    continue;
                }
                let mut page = [0u8; WIN_PAGE_SIZE];
                owner.raw_read(addr, &mut page as *mut u8, WIN_PAGE_SIZE)?;
                let scan_res = owner.scan(pattern, page.iter());

                if let Some(result) = scan_res {
                    println!("Found pattern at {:#x}", scan_res.unwrap());
                    return Ok(Some(addr + result));
                }
                addr += WIN_PAGE_SIZE;
            }
        }
        Ok(None)
    }
    #[cfg(windows)]
    /// scan for a value of <V> in the module
    pub fn scan_value<V>(&self, val: &V) -> Result<Option<usize>, MemError> {
        use windows::Win32::System::Memory::{MEM_COMMIT, PAGE_NOACCESS};
        let mut addr = self.get_base_address();

        while addr < self.get_base_address() + self.get_size() {
            unsafe {
                let query = self.get_owner().raw_query(addr);
                if query.State != MEM_COMMIT || query.Protect == PAGE_NOACCESS {
                    addr += query.RegionSize;
                    continue;
                }
                let mut page = [0u8; WIN_PAGE_SIZE];
                self.get_owner()
                    .raw_read(addr, &mut page as *mut u8, WIN_PAGE_SIZE)?;
                let scan_res = self.get_owner().scan_batch_value(val, &page);

                if let Some(result) = scan_res {
                    println!("Found pattern at {:#x}", scan_res.unwrap());
                    return Ok(Some(addr + result));
                }
                addr += WIN_PAGE_SIZE;
            }
        }
        Ok(None)
    }
}
pub(super) const WIN_PAGE_SIZE: usize = 0x1000;
