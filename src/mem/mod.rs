/// pretty much the main purpose of this crate. 
/// this module is broken into more submodules, external, internal. and other traits which may be used on both.
/// [sigscan] is a trait which allows you to scan for a pattern in a process.
/// [external] is a module which allows you to make external cheats.
/// [internal] is a module which allows you to make internal cheats. 

pub mod traits;
pub mod sigscan;
pub mod external;
pub mod internal;
pub mod structures;
