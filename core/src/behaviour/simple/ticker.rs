use super::{Behaviour, Context, CyclicBehaviour, SimpleBehaviourState, State};

#[cfg(target_os = "none")]
use embassy_time::{Duration, Instant};
#[cfg(not(target_os = "none"))]
use std::time::{Duration, Instant};

pub struct TickerBehaviour<S: SimpleBehaviourState, P> {
    cyclic: CyclicBehaviour<S, P>,
    interval: Duration,
    last_tick: Option<Instant>,
}

impl<S: SimpleBehaviourState, P> TickerBehaviour<S, P> {
    /// Note: Duration in nano seconds must fit in a 64 bit unsigned integer.
    pub fn new(
        interval: core::time::Duration,
        state: S,
        action: impl FnMut(&mut Context, State<S, P>) -> State<S, P> + Send + 'static,
    ) -> Self {
        Self {
            cyclic: CyclicBehaviour::new(state, action),
            interval: from_std_duration(interval),
            last_tick: None,
        }
    }
}

impl<S: SimpleBehaviourState, P> Behaviour for TickerBehaviour<S, P> {
    type ParentState = P;

    fn action(&mut self, ctx: &mut Context, parent_state: P) -> (bool, P) {
        if self
            .last_tick
            .map(|l| Instant::now() - l < self.interval)
            .unwrap_or(false)
        {
            return (false, parent_state);
        }

        self.last_tick = Some(Instant::now());
        self.cyclic.action(ctx, parent_state)
    }
}

fn from_std_duration(duration: core::time::Duration) -> Duration {
    cfg_if::cfg_if! {
        if #[cfg(target_os = "none")] {
            Duration::from_nanos(duration.as_nanos() as u64)
        } else {
            duration
        }
    }
}
