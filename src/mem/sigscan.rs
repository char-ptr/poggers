use anyhow::Result;


pub trait SigScan {
    fn read<T: Default>(&self,addr:usize) -> Result<T>;
    fn scan(&self, pattern: &str,from:usize,size:usize) -> Option<usize> {
        println!("Scanning {:#x} -> {:#x}",from,from+size);
        for i in 0..size {
            let mut okay = true;
            let mut offset = 0;
            for ci in (0..pattern.len()) {
                let c = &pattern[ci..ci+1];
                if c == "?" {
                    offset += 1;
                    continue;
                } else if c == " "{
                    continue;
                } else if ci %3 != 0 {
                    continue;
                }
                let byte = match <Self as SigScan>::read::<u8>(self,from + i + offset){
                    Err(e) => {okay = false;break;}
                    Ok(e) => e
                };
                let byte2 = u8::from_str_radix(&pattern[ci..ci+2].to_string(),16).unwrap();
                if offset > 1 {
                    println!("{:X} -> {:X}",byte,byte2);
                }
                if byte != byte2 {
                    okay = false;
                    break;
                }
                offset += 1;
            }
            if okay {
                println!("pogging");
                return Some(from + i);
            }
        }
        None
    }   
}