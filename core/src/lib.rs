#![cfg_attr(target_os = "none", no_std)]

extern crate alloc;

pub use self::agent::Agent;
pub use self::container::Container;

mod agent;
pub mod behaviour;
mod container;
mod util;
