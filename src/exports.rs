/// this entire module is only used for [poggers_derive::create_entry]
#[cfg(windows)]
pub use windows::{
    Win32::{
        Foundation::{HANDLE,HINSTANCE,BOOL},
        System::{
            SystemServices::DLL_PROCESS_ATTACH,
            Console::{AllocConsole,FreeConsole},
        }
    }
};