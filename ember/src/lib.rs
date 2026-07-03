#![no_std]

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

pub use self::container::Container;

pub mod _crates {
    pub use ember_acc as acc;
    pub use ember_bdi as bdi;
    pub use ember_core as core;
    pub use ember_fipa as fipa;
    pub use ember_reactive as reactive;
}

pub mod agent {
    pub use ember_core::agent::Agent;
    pub use ember_core::agent::aid::Aid;

    pub mod reactive {
        pub use ember_reactive::agent::ReactiveAgent;

        pub use ember_reactive::behaviour;
    }

    pub mod bdi {
        pub use ember_bdi::agent::BdiAgent;
        pub use ember_bdi::{bdi_actions, bdi_agent};

        pub use ember_bdi::{
            bindings, context, event, knowledge, literal, plan, resolve, sensor, term, variable,
        };
    }
}

pub use ember_core::message;

pub use ember_core::environment;

mod adt;
mod container;
