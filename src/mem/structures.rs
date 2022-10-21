use windows::Win32::Foundation::HINSTANCE;
use anyhow::Result;

#[derive(Debug)]
pub struct Process {
    pub(crate) handl: windows::Win32::Foundation::HANDLE,
    pub(crate) pid: u32,
    pub(crate) name: String,
}

#[derive(Debug)]
pub struct Module<'a> {
    pub(crate) process: &'a Process,
    pub(crate) base_address: usize,
    pub(crate) size: usize,
    pub(crate) name: String,
    pub(crate) handle: HINSTANCE,
}

// trait Process {
//     fn from_name(name: &str) -> Result<Self> where Self: Sized;
//     fn get_base_module(&self) -> Result<Box<dyn Module>>;
//     fn get_module(&self, name: &str) -> Result<Box<dyn Module>>;
//     fn get_name(&self) -> &str;
//     fn get_pid(&self) -> u32;
//     fn get_handle(&self) -> windows::Win32::Foundation::HANDLE;
// }

// trait MemEdit {
//     fn read<T>(&self, addr:usize) -> Result<T>;
//     fn read_sized(&self, addr:usize, size:usize) -> Vec<u8>;

//     fn write<T>(&self, addr:usize, val:T) -> Result<()>;
//     fn write_sized(&self, addr:usize, val:Vec<u8>) -> Result<()>;
//     fn scan(&self, sig:&str) -> Result<usize>;
//     fn scan_page(&self, sig:&str, page:Vec<u8>) -> Result<usize>;
//     fn copy<T>(&self, src:usize, dst: *mut T) -> Result<()>;
//     fn copy_sized(&self, src:usize, dst:usize, size:usize) -> Result<()>;
//     fn change_protection<T>(&self, addr:usize, prot:u32) -> Result<()>;
//     fn change_protection_sized(&self, addr:usize,size : usize, prot:u32) -> Result<()>;

// }

// trait Module<'a> {
//     fn new(name: &str, proc: Option<&'a Box<dyn Process>>) -> Result<Self> where Self: Sized;
//     fn get_base_address(&self) -> Result<usize>;
//     fn get_size(&self) -> Result<usize>;
//     fn get_name(&self) -> Result<String>;
//     fn get_handle(&self) -> Result<HINSTANCE>;

// }