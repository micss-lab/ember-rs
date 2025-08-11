use alloc::boxed::Box;

use crate::behaviour::{Behaviour, BehaviourId, IntoBehaviour, get_id};
use crate::context::Context;

pub trait OneShotBehaviour {
    type AgentState;

    type Event;

    fn action(&self, ctx: &mut Context<Self::Event>, agent_state: &mut Self::AgentState);

    fn reset(&mut self) {}
}

struct OneShotBehaviourImpl<O: OneShotBehaviour> {
    id: BehaviourId,
    oneshot: O,
}

impl<O, S, E: 'static> Behaviour for OneShotBehaviourImpl<O>
where
    O: OneShotBehaviour<AgentState = S, Event = E> + 'static,
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
        self.oneshot.action(ctx, agent_state);
        true
    }

    fn reset(&mut self) {
        self.oneshot.reset();
    }
}

#[doc(hidden)]
pub struct OneShot;

impl<T, S, E: 'static> IntoBehaviour<OneShot> for T
where
    T: OneShotBehaviour<AgentState = S, Event = E> + 'static,
{
    type AgentState = S;

    type Event = E;

    fn into_behaviour(self) -> Box<dyn Behaviour<AgentState = S, Event = Self::Event>> {
        Box::new(OneShotBehaviourImpl {
            id: get_id(),
            oneshot: self,
        })
    }
}
