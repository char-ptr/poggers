use crate::traits::Mem;

/// Allocted memory
pub struct VirtAlloc<'a, T : Mem> {
    pub(crate) addr: usize,
    pub(crate) size: usize,
    pub(crate) proc: &'a T
}

impl<'a,T: Mem> VirtAlloc<'a,T> {
    #[cfg(windows)]
    /// Free the allocated memory
    pub fn free(self) {
        self.intrl_free();
    }
    #[cfg(windows)]
    fn intrl_free(&self) {
        unsafe {
            self.proc.raw_virtual_free(self.addr, self.size).ok();
            
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
#[cfg(windows)]
impl<'a,T: Mem> Drop for VirtAlloc<'a,T> {
    #[cfg(windows)]
    fn drop(&mut self) {
        self.intrl_free();
    }
}
