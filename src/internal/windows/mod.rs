/// Windows-specific modules
pub mod module;


/// internal entry point test
#[cfg(test)]
#[cfg(debug_assertions)]
mod test {
    use windows::Win32::Foundation::HMODULE;

    #[poggers_derive::create_entry(no_console)]
    fn test(modil : HMODULE) {
        println!("test");
    }
}