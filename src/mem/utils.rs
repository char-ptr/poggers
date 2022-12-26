/// windows related utilties
#[cfg(windows)]
pub mod windows {

    use windows::core::{PCSTR};
    
    
    /// create a pc string
    #[inline(always)]
    pub fn make_lpcstr(s : &str) -> PCSTR {
        let cstr = format!("{}\0", s);
        PCSTR::from_raw(cstr.as_ptr() as *const u8)
    }
}
#[cfg(windows)]
pub use self::windows::*;