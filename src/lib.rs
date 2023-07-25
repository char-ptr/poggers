//!  # Poggers
//!  A Work in Progress Memory (game cheating) Library for Rust
//!  # Safety
//!  We do not wish to strive away from the safe nature of rust, so all the code in this library **should** be safe if you don't do weird stuff.
//!
//!  # Introduction for windows
//!  ## External
//!  Poggers allows you to effortlessly make external cheats for games. The most important structs & traits are:
//!  * [`ExProcess`] - A wrapper around a process, allowing you to do basic things like read, write and change protections on memory..
//!  * [`ExModule`] - A wrapper around a module.
//!  * [`Mem`] - A trait which implements memory functionality for [`ExProcess`] and [`ExModule`].
//!  * [`SigScan`] - A trait which implements signature scanning functionality for [`ExProcess`] and [`ExModule`].
//!  * [`ToolSnapshot`] - A wrapper around the toolhelp32snapshot api using rust iterators.
//! 
//!  With these two constructs it should make it pretty easy to safe and efficient external cheats.
//!  ## Internal
//!  Poggers also allows you to make internal cheats for games. The most important structs & traits are:
//!  * [`InModule`] - A wrapper around a module.
//!
//!  Check out [poggers-derive](https://crates.io/crates/poggers-derive) for a derive macro to easily run a function upon injection
//!  # Introduction for linux
//!  Not complete.
//!
//!  # Example
//!  ```
//!  use poggers::external::process::ExProcess;
//!  let process = ExProcess::new("notepad.exe").unwrap();
//!  let module = process.base_module().unwrap();
//!  let what = module.scan_virtual("48 8B 05 ? ? ? ? 48 8B 88 ? ? ? ? 48 85 C9 74 0A").unwrap();
//!  process.read::<u32>(what).unwrap();
//!  ```
//!
//!  ## License
//!  This project is licensed under the GPL-2.0 license.



#![warn(missing_docs)]
// #![feature(generic_const_exprs)]
/// exports primarily for [`poggers-derive`]
pub mod exports;

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