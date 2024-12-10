use super::{Behaviour, Context, CyclicBehaviour, SimpleBehaviourState, State};

use core::time::Duration;

#[cfg(target_os = "none")]
use esp_hal::time::Instant;
#[cfg(not(target_os = "none"))]
use std::time::Instant;

pub struct TickerBehaviour<S: SimpleBehaviourState, P> {
    cyclic: CyclicBehaviour<S, P>,
    interval: Duration,
    last_tick: Option<Instant>,
}

impl<S: SimpleBehaviourState, P> TickerBehaviour<S, P> {
    pub fn new(
        interval: Duration,
        state: S,
        action: impl FnMut(&mut Context, State<S, P>) -> State<S, P> + Send + 'static,
    ) -> Self {
        Self {
            cyclic: CyclicBehaviour::new(state, action),
            interval,
            last_tick: None,
        }
    }
}

impl<S: SimpleBehaviourState, P> Behaviour for TickerBehaviour<S, P> {
    type ParentState = P;

    fn action(&mut self, ctx: &mut Context, parent_state: P) -> (bool, P) {
        if self
            .last_tick
            .map(|l| current_time_diff_nanos(l) < self.interval.as_nanos() as u64)
            .unwrap_or(false)
        {
            return (false, parent_state);
        }

        self.last_tick = Some(current_time());
        self.cyclic.action(ctx, parent_state)
    }
}

fn current_time_diff_nanos(last: Instant) -> u64 {
    let current_time = current_time();
    cfg_if::cfg_if! {
        if #[cfg(target_os = "none")] {
            (current_time - last).to_nanos()
        } else {
            (current_time - last).as_nanos() as u64
        }
    }
}

fn current_time() -> Instant {
    cfg_if::cfg_if! {
        if #[cfg(target_os = "none")] {
            esp_hal::time::now()
        } else {
            std::time::Instant::now()
        }
    }
}
