use std::{time::Duration};


static mut STAT_VAL : i32 = 0x7116491; 

fn main() {
    unsafe {
        println!("{:p}",&STAT_VAL as *const i32);
        loop {
            println!("{}",&STAT_VAL);

            // sig scan asm
            let mut test = 0u32;
            // increment stat_val by 1

            std::arch::asm!{
                "add rax, 591754",

                inout("rax") test,
            }

            // timeout 


            std::thread::sleep(Duration::from_millis(300));
        }
    }
}
