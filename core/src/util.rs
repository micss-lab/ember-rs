pub(crate) mod time {
    #[cfg(target_os = "none")]
    pub(crate) use embassy_time::{Duration, Instant};
    #[cfg(not(target_os = "none"))]
    pub(crate) use std::time::{Duration, Instant};

    pub(crate) fn from_std_duration(duration: core::time::Duration) -> Duration {
        cfg_if::cfg_if! {
            if #[cfg(target_os = "none")] {
                Duration::from_nanos(duration.as_nanos() as u64)
            } else {
                duration
            }
        }
    }
}

pub(crate) mod sync {
    use core::cell::Cell;

    #[repr(transparent)]
    pub(crate) struct AtomicU32(Cell<u32>);

    // SAFETY: Internal methods are protected using the [`critical-section`] crate.
    unsafe impl Sync for AtomicU32 {}

    impl AtomicU32 {
        pub(crate) const fn new(value: u32) -> Self {
            Self(Cell::new(value))
        }

        pub(crate) fn get_increment(&self) -> u32 {
            critical_section::with(|_| {
                let value = self.0.get();
                self.0
                    .replace(value.checked_add(1).expect("atomic u32 overflow"))
            })
        }
    }

    #[repr(transparent)]
    pub(crate) struct AtomicBool(Cell<bool>);

    // SAFETY: Internal methods are protected using the [`critical-section`] crate.
    unsafe impl Sync for AtomicBool {}

    impl AtomicBool {
        pub(crate) const fn new(value: bool) -> Self {
            Self(Cell::new(value))
        }

        /// Checks if the inner value is equal to `current` and replaces the value with the
        /// given if they were the same.
        pub(crate) fn compare_and_swap(&self, current: bool, value: bool) -> bool {
            critical_section::with(|_| {
                if self.0.get() == current {
                    self.0.replace(value)
                } else {
                    self.0.get()
                }
            })
        }
    }
}
