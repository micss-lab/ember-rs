use alloc::boxed::Box;

#[cfg(target_os = "none")]
use embassy_time::{Duration, Instant};
#[cfg(not(target_os = "none"))]
use std::time::{Duration, Instant};

pub use self::cyclic::CyclicBehaviour;
pub use self::oneshot::OneShotBehaviour;
pub use self::ticker::TickerBehaviour;

use super::{Behaviour, Context};

mod cyclic;
mod oneshot;
mod ticker;

pub fn oneshot<M: 'static>(
    oneshot: impl OneShotBehaviour<Message = M> + 'static,
) -> Box<dyn Behaviour<Message = M>> {
    Box::new(SimpleBehaviourKind::OneShot(Some(Box::new(oneshot))))
}

pub fn cyclic<M: 'static>(
    cyclic: impl CyclicBehaviour<Message = M> + 'static,
) -> Box<dyn Behaviour<Message = M>> {
    Box::new(SimpleBehaviourKind::Cyclic(Box::new(cyclic)))
}

pub fn ticker<T, M: 'static>(ticker: T) -> Box<dyn Behaviour<Message = M>>
where
    T: TickerBehaviour<Message = M> + 'static,
{
    Box::new(SimpleBehaviourKind::Ticker {
        cyclic: Box::new(ticker),
        interval: from_std_duration(T::interval()),
        last_tick: None,
    })
}

enum SimpleBehaviourKind<M> {
    // Wrapped in an [`Option`] so the user's implementation can use the owned value.
    OneShot(Option<Box<dyn OneShotBehaviour<Message = M>>>),
    Cyclic(Box<dyn CyclicBehaviour<Message = M>>),
    Ticker {
        // Cannot be stored as a [`TickerBehaviour`] because it is not object safe.
        cyclic: Box<dyn CyclicBehaviour<Message = M>>,
        interval: Duration,
        last_tick: Option<Instant>,
    },
}

impl<M: 'static> Behaviour for SimpleBehaviourKind<M> {
    type Message = M;

    fn action(&mut self, ctx: &mut Context<Self::Message>) -> bool {
        match self {
            SimpleBehaviourKind::OneShot(oneshot) => {
                oneshot
                    .take()
                    .expect("oneshot behaviour should only be called once")
                    .action(ctx);
                true
            }
            SimpleBehaviourKind::Cyclic(cyclic) => {
                cyclic.action(ctx);
                cyclic.is_finished()
            }
            SimpleBehaviourKind::Ticker {
                cyclic,
                interval,
                last_tick,
            } => {
                if last_tick
                    .map(|l| Instant::now() - l < *interval)
                    .unwrap_or(false)
                {
                    return cyclic.is_finished();
                }
                *last_tick = Some(Instant::now());
                cyclic.action(ctx);
                cyclic.is_finished()
            }
        }
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
