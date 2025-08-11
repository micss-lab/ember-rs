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

mod adt;
mod agent;
mod container;
mod fipa;

#[cfg(feature = "std")]
mod std_time_driver {
    use ember_core::time::{Driver, Instant, TICK_HZ, time_driver_impl};
    use std::sync::LazyLock;
    use std::time::Instant as StdInstant;

    struct StdDriver(LazyLock<StdInstant>);

    impl Driver for StdDriver {
        fn now(&self) -> Instant {
            Instant::from_ticks(
                StdInstant::now().duration_since(*self.0).as_secs() * TICK_HZ as u64,
            )
        }
    }

    time_driver_impl!(static STD_DRIVER: StdDriver = StdDriver(LazyLock::new(StdInstant::now)));
}
