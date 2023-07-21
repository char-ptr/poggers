use std::{
    fs::OpenOptions,
    io::{BufRead, BufReader},
};

use anyhow::{anyhow, Result};
use libc::{iovec, EFAULT, EINVAL, ENOMEM, EPERM, ESRCH};

use crate::mem::{sigscan::SigScan, traits::Mem};

type Pid = libc::pid_t;

#[derive(Debug)]
pub struct ExProcess {
    pub(crate) pid: Pid,
    pub(crate) name: String,
}
impl ExProcess {
    pub fn new_from_pid(pid: Pid) -> Result<ExProcess> {
        let cmdline = OpenOptions::new()
            .read(true)
            .open(format!("/proc/{}/cmdline", pid))?;
        let contents = BufReader::new(cmdline)
            .lines()
            .next()
            .ok_or(anyhow!("unable to read contents of cmdline"))??;
        let path = std::path::Path::new(
            contents
                .split_whitespace()
                .next()
                .ok_or(anyhow!("unable to get first word of cmdline"))?,
        );
        let name = path
            .file_name()
            .ok_or(anyhow!("unable to get filename from path"))?
            .to_str()
            .ok_or(anyhow!("unable to convert filename to str"))?
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .collect();
        Ok(ExProcess { pid, name })
    }
    pub fn new_from_name(wname: String) -> Result<ExProcess> {
        for entry in std::fs::read_dir("/proc")? {
            let entry = entry?;
            if entry.file_name().to_string_lossy().parse::<Pid>().is_err() {
                continue;
            }
            let cmdline = OpenOptions::new()
                .read(true)
                .open(entry.path().join("cmdline"))?;
            let contents = BufReader::new(cmdline)
                .lines()
                .next()
                .ok_or(anyhow!("unable to read contents of cmdline"))??;
            let path = std::path::Path::new(
                contents
                    .split_whitespace()
                    .next()
                    .ok_or(anyhow!("unable to get first word of cmdline"))?,
            );
            let name = path
                .file_name()
                .ok_or(anyhow!("unable to get filename from path"))?
                .to_str()
                .ok_or(anyhow!("unable to convert filename to str"))?
                .chars()
                .filter(|c| c.is_ascii_alphanumeric())
                .collect();
            if wname == name {
                return Ok(ExProcess {
                    pid: entry
                        .file_name()
                        .to_str()
                        .ok_or(anyhow!("unable to convert filename to str"))?
                        .parse()?,
                    name,
                });
            }
        }
        Err(anyhow!("unable to find process with name {}", wname))
    }
}
impl Mem for ExProcess {
    const PAGE_SIZE: usize = 4096;
    const READ_REQUIRE_PROTECTION: bool = false;
    const WRITE_REQUIRE_PROTECTION: bool = false;
    unsafe fn raw_read(&self, addr: usize, data: *mut u8, size: usize) -> Result<()> {
        let local = libc::iovec {
            iov_base: data as *mut libc::c_void,
            iov_len: size,
        };
        let remote = libc::iovec {
            iov_base: addr as *mut libc::c_void,
            iov_len: size,
        };
        let status = unsafe {
            libc::process_vm_readv(self.pid, &local as *const _, 2, &remote as *const _, 1, 0)
        };
        match status as i32 {
            -1 => Ok(()),
            EINVAL => Err(anyhow!("INVALID ARGUMENTS")),
            EFAULT => Err(anyhow!("UNABLE TO ACCESS TARGET MEMORY ADDRESS")),
            ENOMEM => Err(anyhow!("UNABLE TO ALLOCATE MEMORY")),
            EPERM => Err(anyhow!("INSUFFICIENT PRIVILEGES TO TARGET PROCESS")),
            ESRCH => Err(anyhow!("PROCESS DOES NOT EXIST")),
            _ => Err(anyhow!("UNKNOWN ERROR")),
        }
    }
    unsafe fn raw_write(&self, addr: usize, data: *const u8, size: usize) -> Result<()> {
        let local = libc::iovec {
            iov_base: data as *mut libc::c_void,
            iov_len: size,
        };
        let remote = libc::iovec {
            iov_base: addr as *mut libc::c_void,
            iov_len: size,
        };
        let status = unsafe {
            libc::process_vm_writev(self.pid, &local as *const _, 2, &remote as *const _, 1, 0)
        };
        match status as i32 {
            -1 => Ok(()),
            EINVAL => Err(anyhow!("INVALID ARGUMENTS")),
            EFAULT => Err(anyhow!("UNABLE TO ACCESS TARGET MEMORY ADDRESS")),
            ENOMEM => Err(anyhow!("UNABLE TO ALLOCATE MEMORY")),
            EPERM => Err(anyhow!("INSUFFICIENT PRIVILEGES TO TARGET PROCESS")),
            ESRCH => Err(anyhow!("PROCESS DOES NOT EXIST")),
            _ => Err(anyhow!("UNKNOWN ERROR")),
        }
    }
    unsafe fn alter_protection(
        &self,
        addr: usize,
        size: usize,
        prot: crate::mem::structures::Protections,
    ) -> Result<crate::mem::structures::Protections> {
        Ok(prot)
    }
}
impl SigScan for ExProcess {}
#[derive(Error)]
pub enum ProcessErrors {}
