/// Impementation for linux based systems
#[cfg(target_os = "linux")]
mod linux;


/// Implementation for windows based systems
#[cfg(windows)]
mod windows;


#[cfg(target_os = "linux")]
pub use self::linux::*;
#[cfg(target_os = "windows")]
pub use self::windows::*;