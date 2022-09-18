pub mod mem;
pub mod exports;



pub mod tests {
    #[poggers_derive::create_entry]
    fn deriv_test() -> Result<(),()> {
        println!("pogg");
        Ok(())
    }
    
}