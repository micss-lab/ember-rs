pub use self::cyclic::CyclicBehaviour;
pub use self::oneshot::OneShotBehaviour;
pub use self::ticker::TickerBehaviour;

use super::{Behaviour, Context, IntoBehaviour};
use crate::util::{from_std_duration, Duration, Instant};

mod cyclic;
mod oneshot;
mod ticker;

struct OneShotBehaviourImpl<O: OneShotBehaviour>(Option<O>);
struct CyclicBehaviourImpl<C: CyclicBehaviour>(C);
struct TickerBehaviourImpl<T: TickerBehaviour> {
    ticker: T,
    interval: Duration,
    last_tick: Option<Instant>,
}

impl<M: 'static, O> Behaviour for OneShotBehaviourImpl<O>
where
    O: OneShotBehaviour<Message = M> + 'static,
{
    type Message = M;

    fn action(&mut self, ctx: &mut Context<Self::Message>) -> bool {
        self.0
            .take()
            .expect("oneshot behaviour should only be called once")
            .action(ctx);
        true
    }
}

impl<M: 'static, C> Behaviour for CyclicBehaviourImpl<C>
where
    C: CyclicBehaviour<Message = M> + 'static,
{
    type Message = M;

    fn action(&mut self, ctx: &mut Context<Self::Message>) -> bool {
        self.0.action(ctx);
        self.0.is_finished()
    }
}

impl<M: 'static, T> Behaviour for TickerBehaviourImpl<T>
where
    T: TickerBehaviour<Message = M> + 'static,
{
    type Message = M;

    fn action(&mut self, ctx: &mut Context<Self::Message>) -> bool {
        if self
            .last_tick
            .map(|l| Instant::now() - l < self.interval)
            .unwrap_or(false)
        {
            return self.ticker.is_finished();
        }
        self.last_tick = Some(Instant::now());
        self.ticker.action(ctx);
        self.interval = from_std_duration(self.ticker.interval());
        self.ticker.is_finished()
    }
}
