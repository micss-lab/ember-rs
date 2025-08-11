#![no_std]

extern crate alloc;

pub use ember_core as core;

pub use ember_core::agent::aid::Aid;
pub use ember_core::behaviour;
pub use ember_core::context;
pub use ember_core::message;

pub use self::agent::Agent;
pub use self::container::Container;

mod adt;
mod agent;
mod container;
mod fipa;
