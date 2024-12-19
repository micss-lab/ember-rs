use alloc::boxed::Box;

pub use self::cyclic::CyclicBehaviour;
pub use self::oneshot::OneShotBehaviour;
pub use self::ticker::TickerBehaviour;

use super::{Behaviour, Context, IntoBehaviour};
use crate::util::{from_std_duration, Duration, Instant};

mod cyclic;
mod oneshot;
mod ticker;

enum SimpleBehaviourKind<M> {
    // Wrapped in an [`Option`] so the user's implementation can use the owned value.
    OneShot(Option<Box<dyn OneShotBehaviour<Message = M>>>),
    Cyclic(Box<dyn CyclicBehaviour<Message = M>>),
    Ticker {
        // Cannot be stored as a [`TickerBehaviour`] because it is not object safe.
        ticker: Box<dyn TickerBehaviour<Message = M>>,
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
                ticker,
                interval,
                last_tick,
            } => {
                if last_tick
                    .map(|l| Instant::now() - l < *interval)
                    .unwrap_or(false)
                {
                    return ticker.is_finished();
                }
                *last_tick = Some(Instant::now());
                ticker.action(ctx);
                *interval = from_std_duration(ticker.interval());
                ticker.is_finished()
            }
        }
    }
}
