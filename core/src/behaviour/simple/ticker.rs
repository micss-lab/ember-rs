use alloc::boxed::Box;

use super::{Behaviour, Context, IntoBehaviour, TickerBehaviourImpl};
use crate::util::from_std_duration;

pub trait TickerBehaviour {
    type Message;

    fn interval(&self) -> core::time::Duration;

    fn action(&mut self, ctx: &mut Context<Self::Message>);

    fn is_finished(&self) -> bool;
}

#[doc(hidden)]
pub struct Ticker;

impl<T, M: 'static> IntoBehaviour<Ticker> for T
where
    T: TickerBehaviour<Message = M> + 'static,
{
    type Message = M;

    fn into_behaviour(self) -> Box<dyn Behaviour<Message = Self::Message>> {
        let interval = self.interval();
        Box::new(TickerBehaviourImpl {
            ticker: self,
            interval: from_std_duration(interval),
            last_tick: None,
        })
    }
}
