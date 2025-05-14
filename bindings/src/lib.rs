#![cfg_attr(target_os = "none", no_std)]

extern crate alloc;

mod ffi;
mod log;

#[cfg(target_os = "none")]
mod esp;
