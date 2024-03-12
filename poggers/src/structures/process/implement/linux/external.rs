use libc::{__errno_location, c_void, process_vm_readv, process_vm_writev};
use tracing::instrument;

use crate::{
    sigscan::SigScan,
    structures::{
        process::{implement::utils::ProcessUtils, External, Process, ProcessError, U32OrString},
        protections::Protections,
    },
    traits::Mem,
};

impl Mem for Process<External> {
    /// will always return unsupported.
    #[inline]
    unsafe fn alter_protection(
        &self,
        _addr: usize,
        _size: usize,
        _prot: Protections,
    ) -> Result<Protections, crate::traits::MemError> {
        // let result = mprotect(addr as *mut c_void, size, prot.native());
        // if result == -1 {
        //     Err(crate::traits::MemError::ProtectFailure(addr, size, prot))
        // } else {
        //     Ok(Protections::from_native(result))
        // }
        Err(crate::traits::MemError::Unsupported)
    }

    unsafe fn raw_read(
        &self,
        addr: usize,
        data: *mut u8,
        size: usize,
    ) -> Result<(), crate::traits::MemError> {
        let local = [libc::iovec {
            iov_base: data as *mut libc::c_void,
            iov_len: size,
        }];
        let remote = [libc::iovec {
            iov_base: addr as *mut libc::c_void,
            iov_len: size,
        }];

        let res = process_vm_readv(self.pid as i32, local.as_ptr(), 2, remote.as_ptr(), 1, 0);
        // TODO @pozm:  handle errors;
        println!("res: {}", res);
        let errno = __errno_location().read();
        println!("errno: {}", errno);
        Ok(())
    }

    unsafe fn raw_write(
        &self,
        addr: usize,
        data: *const u8,
        size: usize,
    ) -> Result<(), crate::traits::MemError> {
        let local = [libc::iovec {
            iov_base: data as *mut c_void,
            iov_len: size,
        }];
        let remote = [libc::iovec {
            iov_base: addr as *mut c_void,
            iov_len: size,
        }];

        let res = process_vm_writev(self.pid as i32, local.as_ptr(), 2, remote.as_ptr(), 1, 0);
        // TODO @pozm:  handle errors;

        Ok(())
    }
    /// will always return unsupported.
    #[inline]
    unsafe fn raw_virtual_alloc(
        &self,
        _addr: Option<usize>,
        _size: usize,
        _prot: Protections,
    ) -> Result<usize, crate::traits::MemError> {
        // let addr_or_null = addr.map(|x| x as *mut c_void ).unwrap_or(std::ptr::null_mut());
        // let to_proc = format!("/proc/{}/maps",self.pid);

        // let fder = libc::open(to_proc.as_ptr() as *const i8, libc::O_RDONLY, 0);

        // let addr = libc::mmap(addr_or_null, size, prot.native(), libc::MAP_PRIVATE, fder, 0);

        // Ok(addr as usize)
        Err(crate::traits::MemError::Unsupported)
    }
    /// will always return unsupported.
    #[inline]
    unsafe fn raw_virtual_free(
        &self,
        _addr: usize,
        _size: usize,
    ) -> Result<(), crate::traits::MemError> {
        Err(crate::traits::MemError::Unsupported)
    }
}
impl Process<External> {
    /// find a process by name
    #[instrument]
    pub fn find_by_name(name: &str) -> Result<Self, crate::structures::process::ProcessError> {
        let dir = std::fs::read_dir("/proc/").map_err(|_| {
            ProcessError::UnableToFindProcess(U32OrString::String(name.to_string()))
        })?;
        for file in dir {
            let Ok(file) = file else {
                continue;
            };
            if file
                .file_name()
                .to_string_lossy()
                .chars()
                .any(|x| x.is_alphabetic())
            {
                continue;
            }
            let path = file.path().join("comm");
            let f = std::fs::read_to_string(path).map_err(|_| {
                ProcessError::UnableToFindProcess(U32OrString::String(name.to_string()))
            })?;
            if f.trim() == name {
                let pid = file.file_name().to_str().unwrap().parse::<u32>().unwrap();
                return Self::new(pid);
            }
        }
        Err(ProcessError::UnableToFindProcess(U32OrString::String(
            name.to_string(),
        )))
    }
    /// find a process by pid
    #[instrument]
    pub fn find_by_pid(pid: u32) -> Result<Self, crate::structures::process::ProcessError> {
        let Ok(f) = std::fs::File::open(format!("/proc/{}/comm", pid)) else {
            return Err(ProcessError::UnableToFindProcess(U32OrString::U32(pid)));
        };
        drop(f);
        Self::new(pid)
    }

    #[instrument]
    fn new(pid: u32) -> Result<Self, crate::structures::process::ProcessError> {
        let name = std::fs::read_to_string(format!("/proc/{}/comm", pid))
            .map_err(|_| ProcessError::UnableToFindProcess(U32OrString::U32(pid)))?;
        Ok(Self {
            pid,
            mrk: std::marker::PhantomData,
        })
    }
}
impl ProcessUtils for Process<External> {
    #[instrument]
    fn get_name(&self) -> String {
        std::fs::read_to_string(format!("/proc/{}/comm", self.pid)).unwrap()
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
