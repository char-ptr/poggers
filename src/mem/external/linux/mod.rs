pub mod module;
pub mod process;

#[cfg(test)]
mod tests {
    use crate::mem::external::process::ExProcess;


    #[test]
    fn open_process() {
        let process = ExProcess::new_from_name("nano".to_string()).unwrap();
        println!("Process: {:?}", process);
    }

}
