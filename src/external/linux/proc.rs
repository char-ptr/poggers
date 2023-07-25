use std::fmt::Display;

use thiserror::Error;

pub struct ExLinuxProcess {
    pub pid: libc::pid_t,
    pub name: String,
}

impl ExLinuxProcess {
    pub fn new_from_pid(pid: impl Into<libc::pid_t>) -> Result<Self, ExProcError> {
        let pid = pid.into();
        // to get the name, the process must exist so we skip all checks
        let name = Self::get_name_from_pid(pid)?;
        Ok(Self { pid, name })
    }
    pub fn get_name_from_pid(pid : impl Into<libc::pid_t>) -> Result<String, ExProcError> {
        let pid = pid.into();
        let path = format!("/proc/{}/comm", pid);
        std::fs::read_to_string(path).map_err(|_| ExProcError::ProcessNotFound(U32OrString::U32(pid as u32)))
    }
    pub fn new_from_name(name: impl Into<String>) -> Result<Self, ExProcError> {
        let name = name.into();
        let dirs = std::fs::read_dir("/proc").unwrap();
        for dir in dirs {
            let Ok(dir) = dir else {
                continue;
            };
            let Ok(ft) = dir.file_type() else {
                continue;
            };
            if !ft.is_dir() {
                continue;
            };
            if !dir.file_name().to_string_lossy().chars().all(|x|x.is_numeric()) {
                continue;
            }
            let com = dir.path().join("comm");
            let com = std::fs::read_to_string(com).unwrap();
            if com.trim() == name {
                let pid = dir.file_name().to_string_lossy().parse::<i32>().unwrap();
                return Ok(Self {
                    pid,
                    name,
                });
            }
        }
        return Err(ExProcError::ProcessNotFound(U32OrString::String(name)));
    }
}
#[derive(Debug,Error)]
pub enum ExProcError {
    #[error("process id {0} not found")]
    ProcessNotFound(U32OrString)
}
#[derive(Debug)]
pub enum U32OrString {
    U32(u32),
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


#[cfg(test)]
mod tests {
    use super::ExLinuxProcess;

    #[test]
    fn open_process() {
        let process = ExLinuxProcess::new_from_name("nano".to_string()).unwrap();
        println!("Process: {:?}", process.pid);
    }
}