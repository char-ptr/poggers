use crate::{
    sigscan::SigScan,
    structures::{
        process::{implement::utils::ProcessUtils, External, Internal, Process},
        protections::Protections,
    },
    traits::Mem,
};

impl Mem for Process<Internal> {
    unsafe fn alter_protection(
        &self,
        addr: usize,
        size: usize,
        prot: crate::structures::protections::Protections,
    ) -> Result<crate::structures::protections::Protections, crate::traits::MemError> {
        let result = libc::mprotect(addr as *mut libc::c_void, size, prot.native());
        if result == -1 {
            Err(crate::traits::MemError::ProtectFailure(addr, size, prot))
        } else {
            Ok(Protections::from_native(result))
        }
    }

    unsafe fn raw_read(
        &self,
        addr: usize,
        data: *mut u8,
        size: usize,
    ) -> Result<(), crate::traits::MemError> {
        (addr as *mut u8).copy_to_nonoverlapping(data, size);
        Ok(())
    }

    unsafe fn raw_write(
        &self,
        addr: usize,
        data: *const u8,
        size: usize,
    ) -> Result<(), crate::traits::MemError> {
        (addr as *mut u8).copy_from_nonoverlapping(data, size);
        Ok(())
    }

    unsafe fn raw_virtual_alloc(
        &self,
        addr: Option<usize>,
        size: usize,
        prot: crate::structures::protections::Protections,
    ) -> Result<usize, crate::traits::MemError> {
        let addr_or_null = addr
            .map(|x| x as *mut libc::c_void)
            .unwrap_or(std::ptr::null_mut());
        let to_proc = "/proc/self/maps";

        let fder = libc::open(to_proc.as_ptr() as *const i8, libc::O_RDONLY, 0);

        let addr = libc::mmap(
            addr_or_null,
            size,
            prot.native(),
            libc::MAP_PRIVATE,
            fder,
            0,
        );

        Ok(addr as usize)
    }

    unsafe fn raw_virtual_free(
        &self,
        addr: usize,
        size: usize,
    ) -> Result<(), crate::traits::MemError> {
        libc::munmap(addr as *mut libc::c_void, size);
        Ok(())
    }
}
impl Process<Internal> {
    pub(crate) fn new() -> Self {
        let name = std::fs::read_to_string("/proc/self/comm").unwrap();
        Self {
            pid: unsafe { libc::getpid() } as u32,
            mrk: Default::default(),
        }
    }
}
impl SigScan for Process<Internal> {}
impl ProcessUtils for Process<External> {
    #[instrument]
    fn get_name(&self) -> String {
        std::fs::read_to_string("/proc/self/comm").unwrap()
    }
    #[instrument]
    fn get_module(
        &self,
        name: &str,
    ) -> Result<crate::structures::modules::Module<Self>, crate::structures::modules::ModuleError>
    where
        Self: Sized + SigScan,
    {
        todo!()
    }
}
