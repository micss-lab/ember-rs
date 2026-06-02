#![no_std]

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

pub use self::container::Container;

pub mod _crates {
    pub use ember_acc as acc;
    pub use ember_core as core;
    pub use ember_fipa as fipa;
    pub use ember_reactive as reactive;
}

pub mod agent {
    pub use ember_core::agent::aid::Aid;

    pub mod reactive {
        pub use ember_reactive::agent::*;

        pub use ember_reactive::behaviour;
    }
}

pub use ember_core::message;

mod adt;
mod container;
