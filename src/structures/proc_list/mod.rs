pub mod implement;
struct ProcessList(Box<[ProcessListEntry]>);
struct ProcessListEntry {
    pub pid: u32,
    //pd: PlatformData,
}
impl std::ops::Deref for ProcessListEntry {
    type Target = PlatformData;

    fn deref(&self) -> &Self::Target {
        &self.pd
    }
}
#[cfg(target_os = "macos")]
struct PlatformData;
#[cfg(windows)]
struct PlatformData {
    /// how many threads
    pub thread_count: u32,
    /// the parent process id
    pub parent_id: u32,
    /// the name to the executable
    pub exe_path: String,
}
