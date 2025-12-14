#![no_std]

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

pub use ember_core as core;

pub use ember_core::agent::aid::Aid;
pub use ember_core::behaviour;
pub use ember_core::message;

pub use self::agent::Agent;
pub use self::container::Container;

pub mod channels {
    pub use ember_acc::Acc;
}

mod adt;
mod agent;
mod container;
mod fipa;
