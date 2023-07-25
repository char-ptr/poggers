/// Impementation for linux based systems
#[cfg(target_os = "linux")]
mod linux;


/// Implementation for windows based systems
#[cfg(windows)]
mod windows;


use std::fmt::Display;

#[cfg(target_os = "linux")]
pub use self::linux::*;
#[cfg(target_os = "windows")]
pub use self::windows::*;


/// an enum which can be either a string or a u32
#[derive(Debug)]
pub enum StringOru32 {
    /// a string
    String(String),
    /// a u32
    U32(u32),
}
impl Display for StringOru32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StringOru32::String(s) => write!(f, "{}", s),
            StringOru32::U32(u) => write!(f, "{}", u),
        }
    }
}