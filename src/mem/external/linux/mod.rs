pub mod process;

#[cfg(test)]
mod tests {
    use crate::mem::external::process::ExProcess;


    #[test]
    fn open_process() {
        let process = ExProcess::with_name("nano").unwrap();
        println!("Process: {:?}", process);
    }

}
