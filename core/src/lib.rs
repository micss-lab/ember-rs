#![no_std]

extern crate alloc;

#[cfg(not(target_os = "none"))]
extern crate std;

pub use self::agent::{Agent, Aid};
pub use self::container::Container;

pub mod acc;
mod adt;
mod agent;
mod container;
mod fipa;
