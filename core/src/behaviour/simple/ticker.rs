use alloc::boxed::Box;

use super::{get_id, Behaviour, BehaviourId, Context, IntoBehaviour};
use crate::util::time::{from_std_duration, Duration, Instant};

pub trait TickerBehaviour {
    type AgentState;

    type Event;

    fn interval(&self) -> core::time::Duration;

    fn action(&mut self, ctx: &mut Context<Self::Event>, agent_state: &mut Self::AgentState);

    fn is_finished(&self) -> bool;

    fn reset(&mut self) {}
}

struct TickerBehaviourImpl<T: TickerBehaviour> {
    id: BehaviourId,
    ticker: T,
    interval: Duration,
    last_tick: Option<Instant>,
}

impl<T, S, E: 'static> Behaviour for TickerBehaviourImpl<T>
where
    T: TickerBehaviour<AgentState = S, Event = E> + 'static,
{
    type AgentState = S;

    type Event = E;

    fn id(&self) -> BehaviourId {
        self.id
    }

    fn action(
        &mut self,
        ctx: &mut Context<Self::Event>,
        agent_state: &mut Self::AgentState,
    ) -> bool {
        if self
            .last_tick
            .map(|l| Instant::now() - l < self.interval)
            .unwrap_or(false)
        {
            return self.ticker.is_finished();
        }
        self.last_tick = Some(Instant::now());
        self.ticker.action(ctx, agent_state);
        self.interval = from_std_duration(self.ticker.interval());
        self.ticker.is_finished()
    }

    fn reset(&mut self) {
        self.ticker.reset();
        self.last_tick = None;
    }
}

#[doc(hidden)]
pub struct Ticker;

impl<T, S, E: 'static> IntoBehaviour<Ticker> for T
where
    T: TickerBehaviour<AgentState = S, Event = E> + 'static,
{
    type AgentState = S;

    type Event = E;

    fn into_behaviour(
        self,
    ) -> Box<dyn Behaviour<AgentState = Self::AgentState, Event = Self::Event>> {
        let interval = self.interval();
        Box::new(TickerBehaviourImpl {
            id: get_id(),
            ticker: self,
            interval: from_std_duration(interval),
            last_tick: None,
        })
    }
}
