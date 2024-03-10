#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod mac;
#[cfg(windows)]
mod win32;

/// for internal usage
pub mod utils;

#[cfg(target_os = "linux")]
pub use linux::*;
#[cfg(target_os = "macos")]
pub use mac::*;
#[cfg(windows)]
pub use win32::*;

#[cfg(test)]
mod test {
    fn spawn_test_process() -> std::process::Child {
        use std::process::Command;
        #[cfg(windows)]
        let proc = Command::new("./target/release/rw-test.exe")
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        #[cfg(unix)]
        let proc = Command::new("./target/release/rw-test")
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        proc
    }
    use std::{
        io::{BufRead, BufReader},
        process::Stdio,
    };

    use crate::{structures::process::Process, traits::Mem};

    #[test]
    fn test_reading() {
        let mut proc = spawn_test_process();
        let mut reader = BufReader::new(proc.stdout.take().unwrap());
        let mut bufr = String::new();
        let ex = Process::find_pid(proc.id()).unwrap();

        reader.read_line(&mut bufr).ok();

        println!("bufr: {}", bufr.trim());

        let addr = usize::from_str_radix(bufr.trim_start_matches("0x").trim(), 16).unwrap();

        println!("addr: {:x}", addr);

        let val = unsafe { ex.read::<u32>(addr).unwrap() };

        bufr.clear();

        reader.read_line(&mut bufr).ok();

        println!(
            "val: {}, bufr: {}",
            val,
            bufr.trim().parse::<u32>().unwrap()
        );

        assert_eq!(val, bufr.trim().parse().unwrap());

        proc.kill().unwrap();
    }
    #[test]
    fn test_name_lookup() {
        let mut proc = spawn_test_process();
        let mut reader = BufReader::new(proc.stdout.take().unwrap());
        let mut bufr = String::new();
        #[cfg(windows)]
        let ex = Process::find_name("rw-test.exe").unwrap();
        #[cfg(unix)]
        let ex = Process::find_by_name("rw-test").unwrap();

        reader.read_line(&mut bufr).ok();

        let addr = usize::from_str_radix(bufr.trim_start_matches("0x").trim(), 16).unwrap();

        let val = unsafe { ex.read::<u32>(addr).unwrap() };

        bufr.clear();

        reader.read_line(&mut bufr).ok();

        assert_eq!(val, bufr.trim().parse().unwrap());

        proc.kill().unwrap();
    }
    #[test]
    fn test_writing() {
        let mut proc = spawn_test_process();
        let mut reader = BufReader::new(proc.stdout.take().unwrap());
        let mut bufr = String::new();
        let ex = Process::find_pid(proc.id()).unwrap();

        reader.read_line(&mut bufr).ok();

        let addr = usize::from_str_radix(bufr.trim_start_matches("0x").trim(), 16).unwrap();

        unsafe { ex.write(addr, &4141656).unwrap() };

        reader.read_line(&mut bufr).ok();
        bufr.clear();

        reader.read_line(&mut bufr).ok();

        assert_eq!(4141656, bufr.trim().parse().unwrap());

        proc.kill().unwrap();
    }
}
