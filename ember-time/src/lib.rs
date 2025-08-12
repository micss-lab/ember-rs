//! Idea taken from [embassy-time-driver](https://github.com/embassy-rs/embassy/tree/cae93c5a27d596c9371b81c896598ce1a7fdaa83/embassy-time-driver)

#![no_std]

#[cfg(feature = "std")]
extern crate std;

pub use self::driver::now;

pub const TICK_HZ: u32 = 1_000_000;

pub mod driver;

pub type Instant = fugit::Instant<u64, 1, TICK_HZ>;
pub type Duration = fugit::Duration<u64, 1, TICK_HZ>;
