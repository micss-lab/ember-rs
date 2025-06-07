#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", feature(asm_experimental_arch))]

extern crate alloc;

mod ffi;
mod log;

#[cfg(target_os = "none")]
mod esp;
