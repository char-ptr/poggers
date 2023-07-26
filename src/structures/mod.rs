/// a module in a process
pub mod modules;
/// protections for memory
pub mod protections;
/// process
pub mod process;
/// helper for allocated virtual memory
pub mod virtalloc;

#[cfg(windows)]
/// a wrapper for the win32 snapshottool api
pub mod create_snapshot;