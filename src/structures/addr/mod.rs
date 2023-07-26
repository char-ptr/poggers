use std::rc::Rc;

use crate::{sigscan::SigScan, traits::MemError};

/// represents an address in a process
pub struct Address<T: SigScan> {
    pub(crate) at : usize,
    pub(crate) owner : Rc< T >
}
impl <T: SigScan> Address<T> {
    pub fn new(owner: Rc<T>, at: usize) -> Self {
        Self {
            at,
            owner
        }
    }
    /// Get the address
    pub fn get_address(&self) -> usize {
        self.at
    }
    /// Get the owner of the address
    pub fn get_owner(&self) -> & T {
        self.owner.as_ref()
    }
    /// Read the value at the address
    /// # Safety
    /// This function is unsafe because it can read from any address in the process.
    pub unsafe fn read<V>(&self) -> Result<V,MemError> {
        self.get_owner().read(self.get_address())
    }
    /// Write the value at the address
    /// # Safety
    /// This function is unsafe because it can write to any address in the process.
    pub unsafe fn write<V>(&self, value: &V) -> Result<(),MemError> {
        self.get_owner().write(self.get_address(), &value)
    }
    /// go to an address
    pub fn goto(&mut self, to:usize) {
        self.at = to;
    } 
}
impl <T: SigScan> Clone for Address<T> {
    fn clone(&self) -> Self {
        Self {
            at : self.at,
            owner : self.owner.clone()
        }
    }
}
impl<T:SigScan> std::ops::Add<usize> for Address<T> {
    type Output = Self;
    fn add(mut self, rhs: usize) -> Self::Output {
        self.at += rhs;
        self
    }
}
impl<T:SigScan> std::ops::Div<usize> for Address<T> {
    type Output = Self;
    fn div(mut self, rhs: usize) -> Self::Output {
        self.at /= rhs;
        self
    }
}
impl<T:SigScan> std::ops::Mul<usize> for Address<T> {
    type Output = Self;
    fn mul(mut self, rhs: usize) -> Self::Output {
        self.at *= rhs;
        self
    }
}
impl<T:SigScan> std::ops::Sub<usize> for Address<T> {
    type Output = Self;
    fn sub(mut self, rhs: usize) -> Self::Output {
        self.at -= rhs;
        self
    }
}
