/// wrapper around a address
pub mod addr;
#[feature(modules)]
/// a module in a process
pub mod modules;
#[feature(processes)]
/// an alternative to create_snapshot, just list through all processes running
pub mod proc_list;
#[feature(processes)]
/// process
pub mod process;
/// protections for memory
pub mod protections;
/// helper for allocated virtual memory
pub mod virtalloc;

#[cfg(windows)]
#[feature(snapshot)]
/// a wrapper for the win32 snapshottool api
pub mod create_snapshot;
