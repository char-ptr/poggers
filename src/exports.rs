/// this entire module is only used for [poggers_derive::create_entry]
#[cfg(windows)]
pub use windows::Win32::{
    Foundation::{BOOL, HANDLE},
    System::{
        Console::{AllocConsole, FreeConsole},
        SystemServices::DLL_PROCESS_ATTACH,
    },
};

#[cfg(not(target_os = "windows"))]
pub use ctor::ctor;
