use std::ffi::{c_void, CStr};

#[cfg(target_os = "macos")]
use macos_libproc::proc_name;

#[cfg(not(target_os = "macos"))]
fn main() {}
#[cfg(target_os = "macos")]
fn main() {
    let timer = std::time::Instant::now();
    let buf = [0i32; 1024];
    let ret =
        unsafe { macos_libproc::proc_listallpids(buf.as_ptr() as *mut c_void, buf.len() as i32) };
    let pids: Vec<&i32> = buf.iter().filter(|x| **x != 0).collect();
    // println!("ret: {} : pids= {buf:?}", ret);
    for pid in pids {
        let mut buf = [0u8; 50];
        let _ = unsafe { proc_name(*pid, buf.as_mut_ptr() as *mut c_void, buf.len() as u32) };
        let name = unsafe { CStr::from_ptr(buf.as_mut_ptr() as *mut i8) }
            .to_str()
            .unwrap();
        // println!("pid: {}, name: {:?}", pid, name);
    }
    println!("time: {:?}", timer.elapsed());
}
