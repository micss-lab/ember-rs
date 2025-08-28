#![no_std]
#![cfg(target_os = "none")]

extern crate alloc;

pub mod http;

#[allow(clippy::identity_op)]
const RX_BUFFER_SIZE: usize = 1 * 1024;
#[allow(clippy::identity_op)]
const TX_BUFFER_SIZE: usize = 1 * 1024;
