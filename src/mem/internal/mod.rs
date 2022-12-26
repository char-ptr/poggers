
#[cfg(windows)]
pub mod windows;
pub mod utils;

#[cfg(test)]
pub mod tests {
    use windows::Win32::{System::LibraryLoader::LoadLibraryA, Foundation::GetLastError};

    use crate::mem::utils::make_lpcstr;

    #[test]
    fn find_ntdll () {
        let module = super::windows::module::InModule::new("ntdll.dll").unwrap();
        println!("{:?}", module);
        assert_eq!(module.name, "ntdll.dll");
    }
}