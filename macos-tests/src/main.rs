use std::io::{BufRead, BufReader};
use std::process::Stdio;
use poggers::structures::process::Process;
use poggers::traits::Mem;

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

fn main() {
    test_reading();
}
