use std::{
    ffi::{c_void, CStr, CString},
    mem::zeroed,
};

use libc::{c_char, pid_t};
use macos_libproc::{proc_listpids, PROC_ALL_PIDS};
use tracing::{debug, instrument};

use crate::structures::proc_list::{ProcList, ProcessList};

#[derive(Debug)]
pub struct PlatformData {
    /// the process name (same as in /proc/pid/comm)
    pub proc_name: String,
}
impl ProcList for ProcessList {
    fn get_iter() -> Result<
        impl Iterator<Item = crate::structures::proc_list::ProcessListEntry>,
        crate::structures::proc_list::ProcListError,
    > {
        MacProcessList::new()
    }
}
struct MacProcessList {
    pids: Vec<u32>,
}

impl MacProcessList {
    #[instrument]
    pub fn new() -> Result<Self, super::super::ProcListError> {
        let pids: Vec<u32> = Self::get_pids()?.collect();
        debug!("pid amount = {:?}", pids.len());
        Ok(Self { pids })
    }
    #[instrument]
    pub fn get_pids() -> Result<Box<dyn Iterator<Item = u32>>, super::super::ProcListError> {
        let mut pids = [0 as pid_t; 3072];
        unsafe {
            proc_listpids(
                PROC_ALL_PIDS,
                0,
                pids.as_mut_ptr() as *mut c_void,
                size_of_val(&pids) as i32,
            )
        };
        Ok(Box::new(pids.into_iter().map(|pid| pid as u32)))
    }
}
impl Iterator for MacProcessList {
    type Item = super::super::ProcessListEntry;
    fn next(&mut self) -> Option<Self::Item> {
        let pid = self.pids.pop()?;
        let mut buf = [0u8; 50];
        let _ = unsafe {
            macos_libproc::proc_name(
                pid as i32,
                buf.as_mut_ptr() as *mut c_void,
                buf.len() as u32,
            )
        };
        let pid_name = unsafe { CStr::from_ptr(buf.as_mut_ptr() as *mut i8) }
            .to_str()
            .ok()?;
        Some(super::super::ProcessListEntry {
            pid,
            pd: super::super::PlatformData {
                proc_name: pid_name.to_owned(),
            },
        })
    }
}
