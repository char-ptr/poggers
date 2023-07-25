
#[cfg(windows)]
/// Windows specific internal memory stuff
pub mod windows;
#[cfg(windows)]
#[cfg(test)]
pub mod tests {

    #[test]
    fn find_ntdll () {
        let module = super::windows::module::InModule::new("ntdll.dll").unwrap();
        println!("{:?}", module);
        assert_eq!(module.name, "ntdll.dll");
    }
}