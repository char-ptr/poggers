use std::ffi::CString;

use windows::core::{PCSTR, PCWSTR};

#[inline(always)]
pub fn make_lpcstr(s : &str) -> PCSTR {
    let cstr = format!("{}\0", s);
    PCSTR::from_raw(cstr.as_ptr() as *const u8)
}
#[inline(always)]
pub fn make_pcwstr(s : &str) -> PCWSTR {
    let cstr = CString::new(s).unwrap();
    PCWSTR::from_raw(cstr.as_ptr() as *const u16)
}