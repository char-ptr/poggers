use anyhow::Result;

/// The trait which allows a class to sig scan.
/// * [`Self::scan`] will read for each byte in the size
/// * [`Self::scan_batch`] will take a vector of a page and will not make additional read calls (better for external.)
pub trait SigScan {
    unsafe fn read<T: Default>(&self, addr: usize) -> Result<T>;
    /// Scans for a pattern in the process.
    /// # Arguments
    /// * `pattern` - The pattern to scan for.
    /// * `from` - The address to start scanning from.
    /// * `size` - The size of the region to scan.
    /// # Returns
    /// * [Option<usize>] - The address which has been found.
    unsafe fn scan(&self, pattern: &str, from: usize, size: usize) -> Option<usize> {
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
                let byte = match <Self as SigScan>::read::<u8>(self, from + i + offset) {
                    Err(e) => {
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
    fn scan_batch(&self, pattern: &str, page: &Vec<u8>) -> Option<usize> {
        for i in 0..page.len() {
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
                let byte = page[i + offset];
                let byte2 = u8::from_str_radix(&pattern[ci..ci + 2].to_string(), 16).unwrap();
                if byte != byte2 {
                    okay = false;
                    break;
                }
                offset += 1;
            }
            if okay {
                return Some(i);
            }
        }
        None
    }
}
