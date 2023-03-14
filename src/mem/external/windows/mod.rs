/// a wrapper around the platform specific version.
pub mod process;
/// a wrapper around the platform specific version.
pub mod module;
/// Create a snapshot of the processes or modules
pub mod create_snapshot;

#[cfg(test)]
mod tests {
    use std::{
        io::{BufRead, BufReader},
        process::{Command, Stdio},
    };
    
    use crate::mem::traits::Mem;
    use super::process::*;
    
    #[test]
    fn read_name_search() {
        let proc = Command::new("./test-utils/rw-test.exe")
            .stdout(Stdio::piped())
            .spawn()
            .expect("bruh");
        let l = BufReader::new(proc.stdout.unwrap());
        let mut lines = l.lines();
        let current_val = lines.next().unwrap().unwrap();

        println!("Current value: {}", current_val);
        let xproc = ExProcess::new_from_name("rw-test.exe".to_string()).unwrap();

        let base_mod = xproc.get_base_module().unwrap();

        println!(
            "predicted = {:X} | b = {:X}",
            base_mod.base_address + 0x43000,
            base_mod.base_address
        );

        let offset = base_mod.base_address + 0x43000;
        let read_val = unsafe {xproc.read::<u32>(offset).unwrap()};

        assert_eq!(current_val.parse::<u32>().unwrap(), read_val);
    }
    #[test]
    fn read() {
        let proc = Command::new("./test-utils/rw-test.exe")
            .stdout(Stdio::piped())
            .spawn()
            .expect("bruh");
        let proc_id = proc.id();
        let l = BufReader::new(proc.stdout.unwrap());
        let mut lines = l.lines();
        let current_val = lines.next().unwrap().unwrap();

        println!("Current value: {} -> pid = {proc_id}", current_val);
        let xproc = ExProcess::new_from_pid(proc_id).unwrap();

        let base_mod = xproc.get_base_module().unwrap();

        println!(
            "predicted = {:X} | b = {:X}",
            base_mod.base_address + 0x43000,
            base_mod.base_address
        );

        let offset = base_mod.base_address + 0x43000;
        let read_val = unsafe {xproc.read::<u32>(offset).unwrap()};

        assert_eq!(current_val.parse::<u32>().unwrap(), read_val);
    }
    #[test]
    fn scan_value() {
        let proc = Command::new("./test-utils/rw-test.exe")
            .stdout(Stdio::piped())
            .spawn()
            .expect("bruh");
        let proc_id = proc.id();
        let l = BufReader::new(proc.stdout.unwrap());
        let mut lines = l.lines();
        let current_val = lines.next().unwrap().unwrap();
        let current_val_u32 = current_val.parse::<u32>().unwrap();
        println!("Current value: {} -> pid = {proc_id}", current_val);
        let xproc = ExProcess::new_from_pid(proc_id).unwrap();

        let base_mod = xproc.get_base_module().unwrap();

        println!(
            "predicted = {:X} | b = {:X}",
            base_mod.base_address + 0x43000,
            base_mod.base_address
        );

        let offset = base_mod.base_address + 0x43000;
        let read_offset = unsafe {base_mod.scan_virtual_value(&current_val_u32).unwrap()};
        println!("read_offset = {:X} | predicted => {:X}", read_offset, offset);
        // std::thread::sleep(std::time::Duration::from_secs(45));
        assert_eq!(read_offset, offset);
    }
    #[test]
    fn write() {
        let proc = Command::new("./test-utils/rw-test.exe")
            .stdout(Stdio::piped())
            .spawn()
            .expect("bruh");
        let proc_id = proc.id();
        let l = BufReader::new(proc.stdout.unwrap());
        let mut lines = l.lines();
        let current_val = lines.next().unwrap().unwrap();

        println!("Current value: {} -> pid = {proc_id}", current_val);
        let xproc = ExProcess::new_from_pid(proc_id).unwrap();

        let base_mod = xproc.get_base_module().unwrap();

        println!(
            "predicted = {:X} | b = {:X}",
            base_mod.base_address + 0x43000,
            base_mod.base_address
        );

        let offset = base_mod.base_address + 0x43000;

        let read_val = unsafe {xproc.read::<u32>(offset).unwrap()};

        assert_eq!(current_val.parse::<u32>().unwrap(), read_val);

        unsafe {xproc.write(offset, &9832472u32).unwrap()};

        let current_val = lines.next().unwrap().unwrap();

        assert_eq!(current_val.parse::<u32>().unwrap(), 9832472u32);
    }
    #[test]
    fn sig() {
        let proc = Command::new("./test-utils/rw-test.exe")
            .stdout(Stdio::piped())
            .spawn()
            .expect("bruh");
        let proc_id = proc.id();
        let l = BufReader::new(proc.stdout.unwrap());
        let mut lines = l.lines();
        let current_val = lines.next().unwrap().unwrap();

        println!("Current value: {} -> pid = {proc_id}", current_val);
        let xproc = ExProcess::new_from_pid(proc_id).unwrap();

        let base_mod = xproc.get_base_module().unwrap();

        println!(
            "predicted = {:X} | b = {:X}",
            base_mod.base_address + 0x43000,
            base_mod.base_address
        );

        let offset = unsafe {base_mod.scan_virtual("F3 48 0F 2A C0").unwrap()};
        println!("found @ offset = {:X}", offset);
    }
}
