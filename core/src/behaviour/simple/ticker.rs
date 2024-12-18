use core::ops::DerefMut;

use super::{Context, CyclicBehaviour};

#[cfg(target_os = "none")]
use embassy_time::{Duration, Instant};
#[cfg(not(target_os = "none"))]
use std::time::{Duration, Instant};

pub trait TickerBehaviour {
    type Message;

    fn interval() -> core::time::Duration;

    fn action(&mut self, ctx: &mut Context<Self::Message>);

    fn is_finished(&self) -> bool;
}

pub(crate) struct TickerBehaviourWrapper<C> {
    cyclic: C,
    interval: Duration,
    last_tick: Option<Instant>,
}

impl<C> TickerBehaviourWrapper<C> {
    /// Note: Duration in nano seconds must fit in a 64 bit unsigned integer.
    pub fn new(interval: core::time::Duration, cyclic: C) -> Self {
        Self {
            cyclic,
            interval: from_std_duration(interval),
            last_tick: None,
        }
    }
}

impl<B, C, M> TickerBehaviourWrapper<B>
where
    B: DerefMut<Target = C>,
    C: CyclicBehaviour<Message = M> + ?Sized,
{
    pub(crate) fn action(&mut self, ctx: &mut Context<M>) -> bool {
        if self
            .last_tick
            .map(|l| Instant::now() - l < self.interval)
            .unwrap_or(false)
        {
            return self.cyclic.is_finished();
        }

        self.last_tick = Some(Instant::now());
        self.cyclic.deref_mut().action(ctx);
        self.cyclic.deref().is_finished()
    }
}

impl<T, M> CyclicBehaviour for T
where
    T: TickerBehaviour<Message = M>,
{
    type Message = M;

    fn action(&mut self, ctx: &mut Context<Self::Message>) {
        self.action(ctx)
    }

    fn is_finished(&self) -> bool {
        self.is_finished()
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
