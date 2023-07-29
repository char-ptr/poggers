/// Memory Protection Flags
#[derive(Debug)]
#[cfg(windows)]
pub enum Protections {
    /// If memory can execute in this page ?
    Execute,
    /// If memory can be read and executed
    ExecuteRead,
    /// if memory can be read, write, and executed
    ExecuteReadWrite,
    /// win32 specific
    ExecuteWriteCopy,
    /// No Permissions
    NoAccess,
    /// only read access
    ReadOnly,
    /// can read or write
    ReadWrite,
    /// win32 specific
    WriteCopy,
    /// win32 specific
    TargetInvalid,
    /// win32 specific
    TargerNoUpdate,
    /// invalid protection
    INVALID,
}
/// Memory Protection Flags
#[cfg(target_os = "linux")]
#[bitfield_struct::bitfield(u8)]
pub struct Protections {
    read: bool,
    write: bool,
    execute: bool,
    none: bool,
    #[bits(4)]
    __:u8
}
use std::fmt::{Display, Debug};
#[cfg(windows)]
use windows::Win32::System::Memory::PAGE_PROTECTION_FLAGS;

#[cfg(windows)]
impl Protections {
    /// convert into u32
    pub fn u32(&self) -> u32 {
        match &self {
            Protections::Execute => 0x10,
            Protections::ExecuteRead => 0x20,
            Protections::ExecuteReadWrite => 0x40,
            Protections::ExecuteWriteCopy => 0x80,
            Protections::NoAccess => 0x01,
            Protections::ReadOnly => 0x02,
            Protections::ReadWrite => 0x04,
            Protections::WriteCopy => 0x08,
            Protections::TargetInvalid => 0x40000000,
            Protections::TargerNoUpdate => 0x40000000,
            Protections::INVALID => 0x0,
        }
    }
    /// convert into native version
    pub fn native(&self) -> PAGE_PROTECTION_FLAGS {
        PAGE_PROTECTION_FLAGS(self.u32())
    }
}
#[cfg(windows)]
impl From<u32> for Protections {
    fn from(value: u32) -> Self {
        match value {
            0x10 => Protections::Execute,
            0x20 => Protections::ExecuteRead,
            0x40 => Protections::ExecuteReadWrite,
            0x80 => Protections::ExecuteWriteCopy,
            0x01 => Protections::NoAccess,
            0x02 => Protections::ReadOnly,
            0x04 => Protections::ReadWrite,
            0x08 => Protections::WriteCopy,
            0x40000000 => Protections::TargetInvalid,
            _ => Protections::INVALID,
        }
    }
}
#[cfg(windows)]
impl From<Protections> for u32 {
    fn from(val: Protections) -> Self {
        val.u32()
    }
}
#[cfg(windows)]
impl Display for Protections {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Protections::Execute => write!(f, "Execute"),
            Protections::ExecuteRead => write!(f, "ExecuteRead"),
            Protections::ExecuteReadWrite => write!(f, "ExecuteReadWrite"),
            Protections::ExecuteWriteCopy => write!(f, "ExecuteWriteCopy"),
            Protections::NoAccess => write!(f, "NoAccess"),
            Protections::ReadOnly => write!(f, "ReadOnly"),
            Protections::ReadWrite => write!(f, "ReadWrite"),
            Protections::WriteCopy => write!(f, "WriteCopy"),
            Protections::TargetInvalid => write!(f, "TargetInvalid"),
            Protections::TargerNoUpdate => write!(f, "TargerNoUpdate"),
            Protections::INVALID => write!(f, "INVALID"),
        } 
    }
}

#[cfg(target_os = "linux")]
impl Protections {
    /// get u32 version of the protections
    pub fn u32(&self) -> i32 {
        let mut ret = 0;
        if self.read() {
            ret |= libc::PROT_READ;
        }
        if self.write() {
            ret |= libc::PROT_WRITE;
        }
        if self.execute() {
            ret |= libc::PROT_EXEC;
        }
        if self.none() {
            ret |= libc::PROT_NONE;
        }
        ret
    }
    /// gets the native version of the protections
    pub fn native(&self) -> i32 {
        self.u32()
    }
    /// construct protections from native version
    pub fn from_native(prot : i32) -> Self {
        let mut ret = Protections::new();
        if prot & libc::PROT_READ != 0 {
            ret.set_read(true);
        }
        if prot & libc::PROT_WRITE != 0 {
            ret.set_write(true);
        }
        if prot & libc::PROT_EXEC != 0 {
            ret.set_execute(true);
        }
        if prot == 0 {
            ret.set_none(true);
        }
        ret
    }
}
#[cfg(target_os = "linux")]
impl Display for Protections {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"protections< ")?;
        if self.read() {
            write!(f, "Read ")?;
        }
        if self.write() {
            write!(f, "Write ")?;
        }
        if self.execute() {
            write!(f, "Execute ")?;
        }
        if self.none() {
            write!(f, "None ")?;
        };
        write!(f,">")?;
        Ok(())
    }
}  
#[cfg(test)]
mod tests {
    #[cfg(target_os="linux")]
    #[test]
    fn test_linux_prot() {
        use super::Protections;

        let prot = Protections::new().with_execute(true).with_write(true);
        println!("prot: {}, {}", prot,prot.native());
    }
    #[cfg(target_os="linux")]
    #[test]
    fn test_linux_prot2() {
        use super::Protections;

        let prot = Protections::from_native(6);
        println!("prot: {}", prot);
    }
}