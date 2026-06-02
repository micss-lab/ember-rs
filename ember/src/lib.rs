#![no_std]

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

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
    }
}

mod adt;
mod container;
