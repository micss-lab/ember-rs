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
