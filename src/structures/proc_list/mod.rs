use super::create_snapshot;

/// OS specific implementations of [ProcList]
pub mod implement;
/// a list of processes
#[derive(Debug)]
pub struct ProcessList(Box<[ProcessListEntry]>);
/// a contains basic data about process
#[derive(Debug)]
pub struct ProcessListEntry {
    /// the process id
    pub pid: u32,
    pd: PlatformData,
}
impl std::ops::Deref for ProcessList {
    type Target = [ProcessListEntry];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::Deref for ProcessListEntry {
    type Target = PlatformData;

    fn deref(&self) -> &Self::Target {
        &self.pd
    }
}
#[cfg(target_os = "linux")]
/// standard platform data
pub struct PlatformData;
#[cfg(target_os = "macos")]
/// standard platform data
pub struct PlatformData;
#[cfg(windows)]
type PlatformData = create_snapshot::STProcess;

#[derive(Debug, thiserror::Error)]
/// errors that can occur when getting a process list
pub enum ProcListError {
    /// temp error
    #[error("unable to get process list")]
    UnableToGetProcessList,
}

/// standard functions for getting a process list
pub trait ProcList {
    /// get a [ProcessList] from the current system
    /// under the hood just calls [get_iter] and collects it into a vec
    fn get_list() -> Result<ProcessList, ProcListError> {
        let iter = Self::get_iter()?;
        let hint = iter.size_hint();
        let hint = hint.1.unwrap_or(hint.0);
        let mut vec = Vec::with_capacity(hint);
        for i in iter {
            vec.push(i);
        }
        let proclist = vec.into_boxed_slice();
        Ok(ProcessList(proclist))
    }
    /// get an iterator of [ProcessListEntry] from the current system
    fn get_iter() -> Result<impl Iterator<Item = ProcessListEntry>, ProcListError>;
}
