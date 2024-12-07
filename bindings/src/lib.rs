#![cfg_attr(target_os = "none", no_std)]

extern crate alloc;

mod ffi;

#[cfg(target_os = "none")]
mod esp;
