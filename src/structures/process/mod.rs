/// implementations for process
pub mod implement;

use std::{fmt::Display, marker::PhantomData};

/// represents the process is external
pub struct External;
/// represents the process is internal
pub struct Internal;
/// the type has not yet been determined
pub struct Holding;
/// a process on the operating system
#[derive(Debug, Clone)]
pub struct Process<T = Holding> {
    /// the current process id
    pub(crate) pid: u32,
    #[cfg(windows)]
    /// always none on linux, some on windows. is the handle. (to get actual HANDLE, you must wrap
    /// in HANDLE)
    handl: isize,
    pub(crate) mrk: PhantomData<T>,
}

use crate::sigscan::SigScan;

/// process errors
#[derive(Debug, thiserror::Error)]
pub enum ProcessError {
    /// the process was not found
    #[error("process not found: {0}")]
    UnableToFindProcess(U32OrString),
    /// the process handle could not be opened
    #[error("unable to open process: {0}")]
    UnableToOpenProcess(U32OrString),
    /// unable to get the task even though existing. likely not running as root. This does not need
    /// to be checked outside of macos.
    #[error("unable to get task, are you running as root?")]
    UnableToGetTask,
}
/// Either a u32 or a string
#[derive(Debug)]
pub enum U32OrString {
    /// u32
    U32(u32),
    /// string
    String(String),
}
impl Display for U32OrString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            U32OrString::U32(x) => write!(f, "int {}", x),
            U32OrString::String(x) => write!(f, "str {}", x),
        }
    }
}
impl<T> Process<T> {
    /// get's the pid
    pub fn get_pid(&self) -> u32 {
        self.pid
    }
}
impl Process<Holding> {
    /// get this current process
    pub fn this_process() -> Process<Internal> {
        Process::<Internal>::new()
    }
    /// find a process by id
    pub fn find_pid(pid: u32) -> Result<Process<External>, ProcessError> {
        Process::<External>::try_from(pid)
    }
    /// find a process by name
    pub fn find_name(name: &str) -> Result<Process<External>, ProcessError> {
        Process::<External>::try_from(name)
    }
}
impl TryFrom<u32> for Process<External> {
    type Error = crate::structures::process::ProcessError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::find_by_pid(value)
    }
}
impl TryFrom<&str> for Process<External> {
    type Error = crate::structures::process::ProcessError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::find_by_name(value)
    }
}
impl SigScan for Process<External> {}
trait Proc {
    fn find_by_name(name: &str) -> Result<Process<External>, ProcessError>;
    fn find_by_pid(pid: u32) -> Result<Process<External>, ProcessError>;
}
