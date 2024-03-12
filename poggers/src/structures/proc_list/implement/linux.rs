use tracing::{debug, instrument};

use crate::structures::proc_list::{ProcList, ProcessList};

/// standard platform data
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
        LinuxProcessList::new()
    }
}
struct LinuxProcessList {
    pids: Vec<u32>,
}
impl LinuxProcessList {
    #[instrument]
    pub fn new() -> Result<Self, super::super::ProcListError> {
        let pids: Vec<u32> = Self::get_pids()?.collect();
        debug!("pid amount = {:?}", pids.len());
        Ok(Self { pids })
    }
    #[instrument]
    pub fn get_pids() -> Result<Box<dyn Iterator<Item = u32>>, super::super::ProcListError> {
        let proc_files = std::fs::read_dir("/proc/")
            .map_err(|_| super::super::ProcListError::UnableToGetProcessList)?;
        let pids = proc_files.filter_map(|x| {
            if let Ok(file) = x {
                let name = file.file_name();
                let name = name.to_string_lossy();
                name.parse::<u32>().ok()
            } else {
                None
            }
        });
        Ok(Box::new(pids))
    }
}
impl Iterator for LinuxProcessList {
    type Item = super::super::ProcessListEntry;
    fn next(&mut self) -> Option<Self::Item> {
        let pid = self.pids.pop()?;
        let name = std::fs::read_to_string(format!("/proc/{}/comm", pid))
            .unwrap_or("unknown".to_string())
            .trim()
            .to_string();
        debug!("found process {:?}", name);
        Some(super::super::ProcessListEntry {
            pid,
            pd: super::super::PlatformData { proc_name: name },
        })
    }
}
