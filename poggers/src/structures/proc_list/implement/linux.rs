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
    pub fn new() -> Result<Self, super::super::ProcListError> {
        let mut s = Self { pids: vec![] };
        s.update_pids()?;
        println!("{:?}", s.pids);
        Ok(s)
    }
    pub fn update_pids(&mut self) -> Result<(), super::super::ProcListError> {
        let proc_files = std::fs::read_dir("/proc/")
            .map_err(|_| super::super::ProcListError::UnableToGetProcessList)?;
        for file in proc_files {
            let file = file.map_err(|_| super::super::ProcListError::UnableToGetProcessList)?;
            let name = file.file_name();
            let name = name.to_string_lossy();
            if name.chars().all(|x| x.is_numeric()) {
                let pid = name.parse::<u32>().unwrap();
                self.pids.push(pid);
            }
        }
        Ok(())
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
        Some(super::super::ProcessListEntry {
            pid,
            pd: super::super::PlatformData { proc_name: name },
        })
    }
}
