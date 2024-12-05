#![cfg_attr(target_os = "none", no_std)]

#[cfg(target_os = "none")]
pub mod esp;

#[cfg(not(target_os = "none"))]
pub mod local;
