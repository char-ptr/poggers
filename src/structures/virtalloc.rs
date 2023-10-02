use crate::traits::Mem;

/// Allocted memory
#[must_use = "keep the virtalloc alive to keep the memory allocated"]
pub struct VirtAlloc<'a, T: Mem> {
    pub(crate) addr: usize,
    pub(crate) size: usize,
    pub(crate) proc: &'a T,
}

impl<'a, T: Mem> VirtAlloc<'a, T> {
    /// Free the allocated memory
    pub fn free(self) {
        self.intrl_free();
    }
    fn intrl_free(&self) {
        unsafe {
            self.proc.raw_virtual_free(self.addr, self.size).ok();
        }
    }

    /// Get address of allocated memory
    pub const fn get_addr(&self) -> usize {
        self.addr
    }

    /// Get size of allocated memory
    pub const fn get_size(&self) -> usize {
        self.size
    }
}
impl<'a, T: Mem> Drop for VirtAlloc<'a, T> {
    #[inline(always)]
    fn drop(&mut self) {
        self.intrl_free();
    }
}
