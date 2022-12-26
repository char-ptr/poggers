/* 
# pretty much the main purpose of this crate. 
/// this module is broken into more submodules, external, internal. and other traits which may be used on both.
/// [sigscan] is a trait which allows you to scan for a pattern in a process.
/// [external] is a module which allows you to make external cheats.
/// [internal] is a module which allows you to make internal cheats.
*/

/// Holder of main traits, primarily [`Mem`]
pub mod traits;
/// Holder of the [`SigScan`] trait
pub mod sigscan;
/// For all external related things.
pub mod external;
/// For all internal related things.
pub mod internal;
/// Structures which may be used cross platform.
pub mod structures;
/// Utilities which may be used cross platform.
pub mod utils;
