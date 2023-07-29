use crate::{traits::Mem, structures::process::{Process, Internal}, sigscan::SigScan};

impl Mem for Process<Internal> {
    unsafe fn alter_protection(&self,addr:usize, size: usize, prot: crate::structures::protections::Protections) -> Result<crate::structures::protections::Protections,crate::traits::MemError> {
        todo!()
    }

    unsafe fn raw_read(&self, addr: usize,data: *mut u8, size: usize) -> Result<(),crate::traits::MemError> {
        todo!()
    }

    unsafe fn raw_write(&self, addr: usize,data: *const u8, size: usize) -> Result<(),crate::traits::MemError> {
        todo!()
    }

    unsafe fn raw_virtual_alloc(&self, addr:Option<usize>, size:usize, prot: crate::structures::protections::Protections) -> Result<usize,crate::traits::MemError> {
        todo!()
    }

    unsafe fn raw_virtual_free(&self, addr:usize, size:usize) -> Result<(),crate::traits::MemError> {
        todo!()
    }
}
impl Process<Internal> {
    pub fn new() -> Self {
        todo!()
    }
}
impl Default for Process<Internal> {
    fn default() -> Self {
        Self::new()
    }
}
impl SigScan for Process<Internal> {}
