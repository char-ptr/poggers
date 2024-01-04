pub mod implement;
struct ProcessList(Box<[ProcessListEntry]>);
struct ProcessListEntry {
    pub pid: u32,
    //pd: PlatformData,
}
#[cfg(target_os = "macos")]
struct PlatformData;
