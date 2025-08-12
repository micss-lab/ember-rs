//! Idea taken from [embassy-time-driver](https://github.com/embassy-rs/embassy/tree/cae93c5a27d596c9371b81c896598ce1a7fdaa83/embassy-time-driver)

pub const TICK_HZ: u32 = 1_000_000;

pub type Instant = fugit::Instant<u64, 1, TICK_HZ>;
pub type Duration = fugit::Duration<u64, 1, TICK_HZ>;

pub(crate) fn now() -> Instant {
    unsafe { _ember_time_now() }
}

pub(crate) fn from_core_duration(d: core::time::Duration) -> Duration {
    Duration::from_ticks(libm::round(d.as_secs_f64() * TICK_HZ as f64) as u64)
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
            <$t as $crate::time::Driver>::now(&$name)
        }
    };
}

pub use time_driver_impl;
