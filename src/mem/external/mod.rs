/// Impementation for linux based systems
#[cfg(target_os = "linux")]
pub mod linux;


#[cfg(windows)]
/// Implementation for windows based systems
pub mod windows;