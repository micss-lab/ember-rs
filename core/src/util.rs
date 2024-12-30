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
    pub(crate) struct AtomicU32(u32);

    impl AtomicU32 {
        pub(crate) const fn new(value: u32) -> Self {
            Self(value)
        }

        pub(crate) fn get_increment(&mut self) -> u32 {
            critical_section::with(|_| {
                let result = self.0;
                self.0.checked_add(1).expect("atomic u32 overflow");
                result
            })
        }
    }
}
