/// Impementation for linux based systems
#[cfg(target_os = "linux")]
mod linux;


#[cfg(windows)]
/// Implementation for windows based systems
mod windows;


#[cfg(target_os = "linux")]
pub use linux::*;
#[cfg(target_os = "windows")]
pub use windows::*;