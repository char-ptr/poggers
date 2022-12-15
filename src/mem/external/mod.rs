#[cfg(unix)]
/// Impementation for unix based systems
pub mod unix;


#[cfg(windows)]
/// Implementation for windows based systems
pub mod windows;