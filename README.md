# Poggers
A library for interacting with memory, processes, and similar tools for primarily game cheating purposes.

With poggers you're able to achieve the following things:
- Easily create an entrypoint for your DLL with panic unwrapping already handled for you
- Effortlessly read and write into an external process' memory
- Use IDA signatures to scan for instructions in processes.
- Use CreateToolhelp32Snapshot in a sane fashion with first class support for iterating over processes and modules
- Cross compatibility for Windows and Linux with minimal code changes

[Documentation](https://docs.rs/poggers/latest/poggers/) | [Crates.io](https://crates.io/crates/poggers) | [Repository](https://github.com/pozm/poggers)
## Add into your project
```toml
[dependencies]
poggers = "1"
# if you need to use the entrypoint macro
poggers_derive = "0.1.5"
```

## External Example
a simple example of how you can find a process and write into it's memory
```rust
use poggers::structures::process::Process;
use poggers::traits::Mem;
fn main() {
    let process = Process::find_name("csgo.exe").unwrap();
    unsafe {
        process.write(0x1000,&1).unwrap()
    }
}
```
## Internal Example
If you're making an internal program, you may wish to create an entry point and do stuff with the current process.
```rust
use poggers::structures::process::implement::utils::ProcessUtils;
use poggers::structures::process::Process;
use poggers::traits::Mem;
// automatically create an entry point for your dll! (crate type must be cdylib)
// you also do not need to worry about panic unwinding yourself as it is already done.
#[poggers_derive::create_entry]
fn entry() {
    let this_proc = Process::this_process();
    unsafe {
        let bleh : i32 = this_proc.read(0x1000).unwrap();
        println!("{}",bleh);
        let base_mod_name = this_proc.get_base_module().unwrap().get_name();
        println!("{}",base_mod_name);
    }
}
```

## LICENSE
[this project / library is licensed under **GNU General Public License v2.0**](https://github.com/pozm/poggers/blob/master/LICENSE)
