use anyhow::{Result, Ok};
use thiserror::Error;

pub fn read<T>(addr:usize) -> Result<T> {
    let ptr = addr as *const T;
    let buf :T;
    if ptr.is_null() {
        return Err(InternalError::InvalidPointer(addr).into())
    }
    unsafe {
        buf = ptr.read();
    }
    if std::ptr::addr_of!(buf).is_null() {
        return Err(InternalError::InvalidPointer(addr).into())
    }

    Ok(buf)
}

pub fn read_sized(addr:usize, size:usize) -> Result<Vec<u8>> {
    let mut buffer = vec![0; size];
    let ptr = addr as *const u8;
    if ptr.is_null() {
        return Err(InternalError::InvalidPointer(addr).into())
    }

    unsafe {
        ptr.copy_to_nonoverlapping(buffer.as_mut_ptr(), size);
    }

    Ok(buffer)
}


#[derive(Debug, Error)]
pub enum InternalError {
    #[error("'{0:X}' points to either an invalid address, or a null value")]
    InvalidPointer(usize),
}
