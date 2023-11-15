use std::rc::Rc;

use crate::{sigscan::SigScan, traits::MemError};

/// represents an address in a process
pub struct Address<'a, T: SigScan> {
    at: usize,
    owner: &'a T,
}
impl<'a, T: SigScan> Address<'a, T> {
    /// create a wrapper around this address <at> in <owner>
    pub const fn new(owner: &'a T, at: usize) -> Self {
        Self { at, owner }
    }
    /// Read the value at the address
    /// # Safety
    /// This function is unsafe because it can read from any address in the process.
    pub unsafe fn read<V>(&self) -> Result<V, MemError> {
        self.owner.read(self.at)
    }
    /// Write the value at the address
    /// # Safety
    /// This function is unsafe because it can write to any address in the process.
    pub unsafe fn write<V>(&self, value: &V) -> Result<(), MemError> {
        self.owner.write(self.at, &value)
    }
    /// go to an address
    #[inline(always)]
    pub fn goto(&mut self, to: usize) {
        self.at = to;
    }
}
impl<'a, T: SigScan> Clone for Address<'a, T> {
    fn clone(&self) -> Self {
        Self {
            at: self.at,
            owner: self.owner,
        }
    }
}
impl<'a, T: SigScan> std::ops::Add<usize> for Address<'a, T> {
    type Output = Self;
    fn add(mut self, rhs: usize) -> Self::Output {
        self.at += rhs;
        self
    }
}
impl<'a, T: SigScan> std::ops::Div<usize> for Address<'a, T> {
    type Output = Self;
    fn div(mut self, rhs: usize) -> Self::Output {
        self.at /= rhs;
        self
    }
}
impl<'a, T: SigScan> std::ops::Mul<usize> for Address<'a, T> {
    type Output = Self;
    fn mul(mut self, rhs: usize) -> Self::Output {
        self.at *= rhs;
        self
    }
}
impl<'a, T: SigScan> std::ops::Sub<usize> for Address<'a, T> {
    type Output = Self;
    fn sub(mut self, rhs: usize) -> Self::Output {
        self.at -= rhs;
        self
    }
}
