/// implementations for process
pub mod implement;

use std::{fmt::Display, marker::PhantomData};

/// represents the process is external
pub struct External;
/// represents the process is internal
pub struct Internal;
/// the type has not yet been determined
pub struct Holding;
/// a process on the file system
#[derive(Debug, Clone)]
pub struct Process<T = Holding> {
    /// the current process id
    pub(crate) pid: u32,
    /// always none on linux, some on windows. is the handle.
    #[cfg(windows)]
    pub(crate) handl: isize,
    pub(crate) mrk: PhantomData<T>,
}

#[cfg(windows)]
use windows::Win32::Foundation::HANDLE;
/// base process functions
pub trait ProcessBasics {
    /// get the process id
    fn get_pid(&self) -> u32;
    /// get the process name
    #[cfg(windows)]
    /// get the process handle [WINDOWS ONLY]
    fn get_handle(&self) -> HANDLE;
}

impl<T> ProcessBasics for Process<T> {
    /// get the process id
    fn get_pid(&self) -> u32 {
        self.pid
    }
    /// get the process handle
    /// WINDOWS ONLY
    #[cfg(windows)]
    fn get_handle(&self) -> HANDLE {
        HANDLE(self.handl)
    }
}

/// process errors
#[derive(Debug, thiserror::Error)]
pub enum ProcessError {
    /// the process was not found
    #[error("process not found: {0}")]
    UnableToFindProcess(U32OrString),
    /// the process handle could not be opened
    #[error("unable to open process: {0}")]
    UnableToOpenProcess(U32OrString),
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
