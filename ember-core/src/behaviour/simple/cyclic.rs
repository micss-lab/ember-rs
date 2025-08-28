use alloc::boxed::Box;

use crate::behaviour::{Behaviour, BehaviourId, IntoBehaviour, get_id};
use crate::context::Context;

pub trait CyclicBehaviour {
    type AgentState;

    type Event;

    fn action(&mut self, ctx: &mut Context<Self::Event>, agent_state: &mut Self::AgentState);

    fn is_finished(&self) -> bool;

    fn reset(&mut self) {}
}

struct CyclicBehaviourImpl<C: CyclicBehaviour> {
    id: BehaviourId,
    cyclic: C,
}

impl<S, E, C> Behaviour for CyclicBehaviourImpl<C>
where
    C: CyclicBehaviour<AgentState = S, Event = E>,
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
        self.cyclic.action(ctx, agent_state);
        self.cyclic.is_finished()
    }

    fn reset(&mut self) {
        self.cyclic.reset();
    }
}

#[doc(hidden)]
pub struct Cyclic;

impl<'a, T, S, E> IntoBehaviour<'a, Cyclic> for T
where
    T: CyclicBehaviour<AgentState = S, Event = E> + 'a,
{
    type AgentState = S;

    type Event = E;

    fn into_behaviour(
        self,
    ) -> Box<dyn Behaviour<AgentState = Self::AgentState, Event = Self::Event> + 'a> {
        Box::new(CyclicBehaviourImpl {
            id: get_id(),
            cyclic: self,
        })
    }
}
