use super::traits::Mem;
/// The trait which allows a class to sig scan.
/// # Notes
/// Requires the [`Mem`] trait to be implemented.
/// # Functions
/// * [`Self::scan`] will read for each byte in the size
/// * [`Self::scan_batch`] will take a vector of a page and will not make additional read calls (better for external.)
pub trait SigScan: Mem {

    /// Scans for a pattern in the process.
    /// # Arguments
    /// * `pattern` - The pattern to scan for.
    /// * `from` - The address to start scanning from.
    /// * `size` - The size of the region to scan.
    /// # Returns
    /// * [Option<usize>] - The address which has been found.
    fn scan<'a>(&self, pattern:&str, iter : impl Iterator<Item = &'a u8>) ->  Option<usize> {

        let mut compiled_pattern : Vec<u8> = Vec::with_capacity(pattern.len());
        let mut skip = false;
        for (i,char) in pattern.chars().enumerate() {
            if skip {
                skip = false;
                continue;
            }
            match char {
                '?' => compiled_pattern.push(b'?'),
                ' ' => continue,
                _ => {
                    let charpt = &pattern[i..i+2];
                    let byte = u8::from_str_radix(charpt, 16).unwrap();
                    compiled_pattern.push(byte);
                    skip = true;
                }
            }
        }
        let mut pi = 0;
        let pl = compiled_pattern.len();
        for (i,data) in iter.enumerate() {
            match compiled_pattern[pi] {
                b'?' => {
                    pi += 1;
                    continue;
                }
                _ => {}
            }
            if *data == compiled_pattern[pi] {
                if pi == pl - 1 {
                    return Some(i - pl + 1);
                }
                pi += 1;
            } else {
                pi = 0;
            }
        }
        None
    }

    /// Scans for a pattern in the process.
    /// # Arguments
    /// * `pattern` - The pattern to scan for.
    /// * `from` - The address to start scanning from.
    /// * `size` - The size of the region to scan.
    /// # Returns
    /// * [Option<usize>] - The address which has been found.
    unsafe fn oscan(&self, pattern: &str, from: usize, size: usize) -> Option<usize> {
        for i in 0..size {
            let mut okay = true;
            let mut offset = 0;
            for ci in 0..pattern.len() {
                let c = &pattern[ci..ci + 1];
                if c == "?" {
                    offset += 1;
                    continue;
                } else if c == " " {
                    continue;
                } else if ci % 3 != 0 {
                    continue;
                }
                let byte = match Self::read::<u8>(self, from + i + offset) {
                    Err(_) => {
                        okay = false;
                        break;
                    }
                    Ok(e) => e,
                };
                let byte2 = u8::from_str_radix(&pattern[ci..ci + 2].to_string(), 16).unwrap();
                if byte != byte2 {
                    okay = false;
                    break;
                }
                offset += 1;
            }
            if okay {
                return Some(from + i);
            }
        }
        None
    }
    /// Scans for a pattern in a page.
    /// # Arguments
    /// * `pattern` - The pattern to scan for.
    /// * `page` - The page to scan.
    /// # Returns
    /// * [Option<usize>] - The address which has been found.
    fn scan_batch(&self, pattern: &str, page: &[u8]) -> Option<usize> {
        for i in 0..page.len() {

            let mut okay = true;
            let mut offset = 0;
            let mut skip_next = false;

            for ci in 0..pattern.len() {
                let c = &pattern[ci..ci + 1];
                if c == "?" {
                    offset += 1;
                    continue;
                } else if c == " " {
                    continue;
                } else if skip_next {
                    skip_next = false;
                    continue;
                }
                let byte = page.get(i + offset);
                if byte.is_none() {
                    okay = false;
                    break;
                }
                let byte = byte.unwrap();
                let byte2 = u8::from_str_radix(&pattern[ci..ci + 2].to_string(), 16).unwrap();
                if *byte != byte2 {
                    okay = false;
                    break;
                }
                offset += 1;
                skip_next = true;
            }
            if okay {
                return Some(i);
            }
        }
        None
    }
    /// scan for a value of <T>
    fn scan_batch_value<T : Sized>(&self, val: &T, page: &[u8]) -> Option<usize> {
        let type_size = std::mem::size_of::<T>();
        let mut val_arr = vec![0; type_size];
        unsafe {(val as *const T as *const u8).copy_to_nonoverlapping(val_arr.as_mut_ptr(), type_size) };
        for (i,val) in page.chunks(type_size).enumerate() {
            // println!("val in mem :{:X?} - looking for: {:X?}", &val, &val_arr);
            if val == val_arr {
                return Some(i * type_size);
            }
        }
        None
    }
}
