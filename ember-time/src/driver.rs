use crate::Instant;

pub fn now() -> Instant {
    unsafe { _ember_time_now() }
}

pub trait Driver: Send + Sync + 'static {
    fn now(&self) -> Instant;
}

unsafe extern "Rust" {
    fn _ember_time_now() -> Instant;
}

#[macro_export]
macro_rules! time_driver_impl {
    (static $name:ident: $t: ty = $val:expr) => {
        static $name: $t = $val;

        #[unsafe(no_mangle)]
        fn _ember_time_now() -> Instant {
            <$t as $crate::driver::Driver>::now(&$name)
        }
    };
}

pub use time_driver_impl;

#[cfg(feature = "std")]
mod std_time_driver_impl {
    use std::sync::LazyLock;
    use std::time::Instant as StdInstant;

    use super::{Driver, Instant, time_driver_impl};
    use crate::TICK_HZ;

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

/// Stolen from the [esp-hal] crate.
///
/// [esp-hal]: https://github.com/esp-rs/esp-hal/blob/713cd491b6a6645bc8fe107d1e4d284135ca4459/esp-hal/src/time.rs
#[cfg(feature = "esp32")]
pub mod esp32_tim_driver_impl {
    use esp32::TIMG0;

    use super::{Driver, time_driver_impl};
    use crate::Instant;

    struct Esp32Driver;

    impl Driver for Esp32Driver {
        fn now(&self) -> Instant {
            let (ticks, div) = {
                // on ESP32 use LACT
                let tg0 = unsafe { TIMG0::steal() };
                tg0.lactupdate().write(|w| unsafe { w.update().bits(1) });

                // The peripheral doesn't have a bit to indicate that the update is done, so we
                // poll the lower 32 bit part of the counter until it changes, or a timeout
                // expires.
                let lo_initial = tg0.lactlo().read().bits();
                let mut div = tg0.lactconfig().read().divider().bits();
                let lo = loop {
                    let lo = tg0.lactlo().read().bits();
                    if lo != lo_initial || div == 0 {
                        break lo;
                    }
                    div -= 1;
                };
                let hi = tg0.lacthi().read().bits();

                let ticks = (hi as u64) << 32u64 | lo as u64;
                (ticks, 16)
            };

            Instant::from_ticks(ticks / div)
        }
    }

    time_driver_impl!(static ESP32_DRIVER: Esp32Driver = Esp32Driver);
}
