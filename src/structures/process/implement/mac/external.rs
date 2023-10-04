use std::{ffi::CStr, marker::PhantomData};

use libc::{c_void, mach_task_self, vm_address_t};
use mach::{
    kern_return::KERN_SUCCESS,
    traps::{task_for_pid},
    vm_statistics::VM_FLAGS_ANYWHERE,
    vm_types::{mach_vm_address_t, mach_vm_size_t},
};
use mach::port::MACH_PORT_NULL;
use mach::vm_inherit::VM_INHERIT_NONE;

use crate::{
    structures::process::{External, Process, ProcessError, U32OrString},
    traits::Mem,
};
use crate::traits::MemError::WriteFailure;

// for any future maintainer : https://web.mit.edu/darwin/src/modules/xnu/osfmk/man/vm_read.html. you'll thank me
// https://opensource.apple.com/source/xnu/xnu-4570.1.46/libsyscall/mach/mach_vm.c.auto.html
// https://github.com/apple/darwin-xnu/blob/main/libsyscall/mach/mach_vm.c
impl Mem for Process<External> {
    unsafe fn alter_protection(
        &self,
        addr: usize,
        size: usize,
        prot: crate::structures::protections::Protections,
    ) -> Result<crate::structures::protections::Protections, crate::traits::MemError> {
        let task = self.task().ok_or(ProcessError::UnableToGetTask)?;

        let ret = mach::vm::mach_vm_map(
            task,
            addr as *mut mach_vm_address_t,
            size as mach_vm_size_t,
            0,
            VM_FLAGS_ANYWHERE,
            MACH_PORT_NULL,
            0,
            0,
            prot.native(),
            prot.native(),
            VM_INHERIT_NONE,

        );
        println!("kernel responded with {ret}");
        Ok(prot)
    }
    unsafe fn raw_read(
        &self,
        addr: usize,
        data: *mut u8,
        size: usize,
    ) -> Result<(), crate::traits::MemError> {
        let mut sz = std::mem::zeroed();
        let task = self.task().ok_or(ProcessError::UnableToGetTask)?;
        let ret = mach::vm::mach_vm_read_overwrite(
            task,
            addr as mach_vm_address_t,
            size as mach_vm_size_t,
            data as mach_vm_address_t,
            &mut sz,
        );
        if ret != KERN_SUCCESS {
            return Err(crate::traits::MemError::ReadFailure(addr));
        }
        Ok(())
    }
    unsafe fn raw_write(
        &self,
        addr: usize,
        data: *const u8,
        size: usize,
    ) -> Result<(), crate::traits::MemError> {
        let task = self.task().ok_or(ProcessError::UnableToGetTask)?;
        let ret = mach::vm::mach_vm_write(task, addr as u64, data as vm_address_t, size as u32);

        if ret != KERN_SUCCESS {
            return Err(WriteFailure(addr))
        }
        Ok(())
    }
    unsafe fn raw_virtual_alloc(
        &self,
        addr: Option<usize>,
        size: usize,
        _: crate::structures::protections::Protections,
    ) -> Result<usize, crate::traits::MemError> {
        let addr_ptr = addr.map(|x| x as *mut u64).unwrap_or(std::ptr::null_mut());
        let task = self.task().ok_or(ProcessError::UnableToGetTask)?;
        let ret = mach::vm::mach_vm_allocate(task, addr_ptr, size as u64, VM_FLAGS_ANYWHERE);
        if ret != KERN_SUCCESS {
            return Err(crate::traits::MemError::AllocFailure(addr,size));
        }
        Ok(addr_ptr as usize)
    }
    unsafe fn raw_virtual_free(
        &self,
        addr: usize,
        size: usize,
    ) -> Result<(), crate::traits::MemError> {
        let ret = mach::vm::mach_vm_deallocate(self.pid, addr as u64, size as u64);

        println!("kernel responded with {ret}");
        Ok(())
    }
}

impl Process<External> {

    /// iterates through all processed to make sure your pid is valid.
    /// this internally uses proc_listallpids, which is a kernel function.
    /// and requires a buffer input. this buffer is 1024 i32's long.
    /// if you have more than 1024 processes running, this could fail.
    pub fn find_by_pid(pid: u32) -> Result<Self, ProcessError> {
        let mut buf = [0i32; 1024];
        let ret = unsafe {
            macos_libproc::proc_listallpids(buf.as_mut_ptr() as *mut c_void, buf.len() as i32)
        };
        if ret != 256 {
            return Err(ProcessError::UnableToFindProcess(U32OrString::U32(pid)));
        }
        let _ = buf
            .iter()
            .find(|x| x == &&(pid as i32))
            .ok_or(ProcessError::UnableToFindProcess(U32OrString::U32(pid)))?;

        Ok(Process::<External> {
            pid,
            mrk: PhantomData,
        })
    }
    /// gets the task for the process
    pub fn task(&self) -> Option<u32> {
        let mut task: u32 = 0;
        let current_task = unsafe { mach_task_self() };
        let ret = unsafe { task_for_pid(current_task, self.pid as i32, &mut task) };
        if ret != KERN_SUCCESS {
            return None;
        }
        return Some(task);
    }
    /// finds the process from a name
    pub fn find_by_name(name: &str) -> Result<Self, ProcessError> {
        let mut buf = [0i32; 1024];
        let ret = unsafe {
            macos_libproc::proc_listallpids(buf.as_mut_ptr() as *mut c_void, buf.len() as i32)
        };
        if ret != 256 {
            return Err(ProcessError::UnableToFindProcess(U32OrString::String(
                name.to_string(),
            )));
        };

        let pid = buf
            .iter()
            .find(|x| {
                let mut buf = [0u8; 50];
                let _ = unsafe {
                    macos_libproc::proc_name(**x, buf.as_mut_ptr() as *mut c_void, buf.len() as u32)
                };
                let pid_name = unsafe { CStr::from_ptr(buf.as_mut_ptr() as *mut i8) }
                    .to_str()
                    .unwrap();
                pid_name == name
            })
            .ok_or(ProcessError::UnableToFindProcess(U32OrString::String(
                name.to_string(),
            )))?;
        Ok(Process::<External> {
            pid: *pid as u32,
            mrk: PhantomData,
        })
    }
}
