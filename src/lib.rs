//!  # Poggers
//!  A library for interacting with memory, processes, and similar tools for primarily game cheating purposes.
//! With poggers you're able to achieve the following things:
//! - Easily create an entrypoint for your DLL with panic unwrapping already handled for you
//! - Effortlessly read and write into an external process' memory
//! - Use IDA signatures to scan for instructions in processes.
//! - Use CreateToolhelp32Snapshot in a sane fashion with first class support for iterating over processes and modules
//! - Cross compatibility for Windows and Linux with minimal code changes
//!
//! [Documentation](https://docs.rs/poggers/latest/poggers/) | [Crates.io](https://crates.io/crates/poggers) | [Repository](https://github.com/pozm/poggers)
//! ## Add into your project
//! ```toml
//! [dependencies]
//! poggers = "1"
//! # if you need to use the entrypoint macro
//! poggers_derive = "0.1.5"
//! ```
//!  ## Common Structs
//!  * [`Process`](structures::process::Process) - A struct which holds the handle to a process.
//!  * [`Module`](structures::modules::Module) - A struct which holds the handle to a module.
//!  * [`ToolSnapshot`](structures::create_snapshot::ToolSnapshot) - A wrapper around the ToolHelp32Snapshot function.
//!  ## Common Traits
//!  * [`Mem`](traits::Mem) - A trait which allows a struct to read and write to memory.
//!  * [`SigScan`](sigscan::SigScan) - A trait which allows a struct to sig scan.
//!  ## Example External usage:
//! ```
//!  use poggers::structures::process::Process;
//!  use poggers::traits::Mem;
//!  fn main() {
//!     let process = Process::find_name("csgo.exe").unwrap();
//!     unsafe {
//!         process.write(0x1000,&1).unwrap()
//!     }
//! }
//! ```
//!  ## Example Internal usage:
//! ```
//! use poggers::structures::process::implement::utils::ProcessUtils;
//! use poggers::structures::process::Process;
//! use poggers::traits::Mem;
//! // automatically create an entry point for your dll! (crate type must be cdylib)
//! // you also do not need to worry about panic unwinding yourself as it is already done.
//! #[poggers_derive::create_entry]
//! fn entry() {
//!     let this_proc = Process::this_process();
//!     unsafe {
//!         let bleh : i32 = this_proc.read(0x1000).unwrap();
//!         println!("{}",bleh);
//!         let base_mod_name = this_proc.get_base_module().unwrap().get_name();
//!         println!("{}",base_mod_name);
//!     }
//! }
//! ```
//!  ## License
//!  This project is licensed under the GPL-2.0 license.

#![warn(missing_docs)]

// #![feature(generic_const_exprs)]
/// exports primarily for [`poggers-derive`]
pub mod exports;

/// Holder of the [`SigScan`](sigscan::SigScan) trait
pub mod sigscan;
/// Structures which may be used cross platform.
pub mod structures;
/// Holder of main traits, primarily [`Mem`](traits::Mem)
pub mod traits;
// struct Temp;
