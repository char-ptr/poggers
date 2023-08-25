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
    fn scan<'a>(&self, pattern: &str, iter: impl Iterator<Item = &'a u8>) -> Option<usize> {
        let mut compiled_pattern: Vec<u8> = Vec::with_capacity(pattern.len());
        let mut skip = false;
        for (i, char) in pattern.chars().enumerate() {
            if skip {
                skip = false;
                continue;
            }
            match char {
                '?' => compiled_pattern.push(b'?'),
                ' ' => continue,
                _ => {
                    let charpt = &pattern[i..i + 2];
                    let byte = u8::from_str_radix(charpt, 16).unwrap();
                    compiled_pattern.push(byte);
                    skip = true;
                }
            }
        }
        let mut pattern_index = 0;
        let pattern_length = compiled_pattern.len();
        for (i, data) in iter.enumerate() {
            let pattern_byte = compiled_pattern.get(pattern_index);
            match pattern_byte {
                Some(b'?') => {
                    if pattern_index == pattern_length - 1 {
                        return Some(i - pattern_length + 1);
                    }
                    pattern_index += 1;
                    continue;
                }
                Some(x) if x == data => {
                    if pattern_index == pattern_length - 1 {
                        return Some(i - pattern_length + 1);
                    }
                    pattern_index += 1;
                }
                _ => {
                    pattern_index = 0;
                }
            }
        }
        None
    }
    /// scans for a value in a page
    fn scan_batch_value<T: Sized>(&self, val: &T, page: &[u8]) -> Option<usize> {
        let type_size = std::mem::size_of::<T>();
        let mut val_arr = vec![0; type_size];
        unsafe {
            (val as *const T as *const u8).copy_to_nonoverlapping(val_arr.as_mut_ptr(), type_size)
        };
        for (i, val) in page.chunks(type_size).enumerate() {
            // println!("val in mem :{:X?} - looking for: {:X?}", &val, &val_arr);
            if val == val_arr {
                return Some(i * type_size);
            }
        }
        None
    }
}
