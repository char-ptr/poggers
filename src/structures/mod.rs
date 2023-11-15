/// wrapper around a address
pub mod addr;
/// a module in a process
pub mod modules;
/// an alternative to create_snapshot, just list through all processes running
pub mod proc_list;
/// process
pub mod process;
/// protections for memory
pub mod protections;
/// helper for allocated virtual memory
pub mod virtalloc;

#[cfg(windows)]
/// a wrapper for the win32 snapshottool api
pub mod create_snapshot;

