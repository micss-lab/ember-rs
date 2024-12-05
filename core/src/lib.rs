#![cfg_attr(not(test), no_std)]

extern crate alloc;

pub use self::agent::Agent;
pub use self::container::Container;

mod agent;
pub mod behaviour;
mod container;
