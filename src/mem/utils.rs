#[macro_export]
macro_rules! make_pcstr {
    ($str:expr) => {
        PCSTR::from_raw(CString::new($str).unwrap().as_ptr() as *const u8)
    };
}