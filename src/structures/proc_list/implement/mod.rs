#[cfg(target_os = "macos")]
mod mac;

#[cfg(target_os = "macos")]
pub use mac::*;

#[cfg(target_os = "windows")]
mod win32;

#[cfg(target_os = "windows")]
pub use win32::*;

#[cfg(test)]
mod tests {
    use crate::structures::proc_list::{ProcList, ProcessList};

    #[test]
    fn test_list() {
        let list = ProcessList::get_list().unwrap();
        println!("{:#?}", list);
    }
}
