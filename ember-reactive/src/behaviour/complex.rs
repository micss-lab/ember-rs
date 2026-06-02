use crate::behaviour::{Behaviour, BehaviourId, IntoBehaviour, get_id};
use crate::context::Context;

use self::scheduler::BehaviourScheduler;

pub mod fsm;
pub mod parallel;
pub mod sequential;

pub mod blocked;
pub mod scheduler;

pub trait ComplexBehaviour {
    type Event;

    type ChildEvent;

    type AgentState;

    fn handle_child_event(&mut self, event: Self::ChildEvent) {
        let _ = event;
    }

    fn after_child_action(
        &mut self,
        ctx: &mut Context<Self::Event>,
        agent_state: &mut Self::AgentState,
    ) {
        let _ = ctx;
        let _ = agent_state;
    }

    fn reset(&mut self) {}
}

trait ScheduledComplexBehaviour<'a>: ComplexBehaviour {
    fn scheduler(&mut self)
    -> &mut impl BehaviourScheduler<'a, Self::AgentState, Self::ChildEvent>;
}

struct ComplexBehaviourImpl<I> {
    id: BehaviourId,
    inner: I,
}

impl<'a, I, S, E, CE> Behaviour for ComplexBehaviourImpl<I>
where
    I: ScheduledComplexBehaviour<'a, AgentState = S, Event = E, ChildEvent = CE>,
{
    type Event = E;

    type AgentState = S;

    fn id(&self) -> BehaviourId {
        self.id
    }

    fn action(
        &mut self,
        context: &mut Context<Self::Event>,
        agent_state: &mut Self::AgentState,
    ) -> bool {
        let mut child_context = context.fresh_local();

        // 1. Execute next scheduled behaviour.
        self.inner
            .scheduler()
            .action(&mut child_context, agent_state);

        // 2. Handle events the behaviour produced.
        while let Some(event) = child_context.local.events.pop() {
            self.inner.handle_child_event(event);
        }

        // 3. Run user defined actions for this complex behaviour.
        self.inner.after_child_action(context, agent_state);

        self.inner.scheduler().is_finished()
    }

    fn reset(&mut self) {
        self.inner.reset();
    }
}
