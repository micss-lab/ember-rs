use crate::behaviour::{Behaviour, BehaviourId, IntoBehaviour, get_id};
use crate::context::Context;

use self::scheduler::BehaviourScheduler;

pub mod blocked;
pub mod fsm;
pub mod parallel;

pub mod scheduler;
pub mod sequential;

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

trait ScheduledComplexBehaviour: ComplexBehaviour
where
    Self::AgentState: 'static,
    Self::ChildEvent: 'static,
{
    fn scheduler(&mut self) -> &mut impl BehaviourScheduler<Self::AgentState, Self::ChildEvent>;
}

struct ComplexBehaviourImpl<I> {
    id: BehaviourId,
    inner: I,
}

impl<I, S: 'static, E: 'static, CE: 'static> Behaviour for ComplexBehaviourImpl<I>
where
    I: ScheduledComplexBehaviour<AgentState = S, Event = E, ChildEvent = CE> + 'static,
{
    type Event = E;

    type AgentState = S;

    fn id(&self) -> BehaviourId {
        self.id
    }

    fn action(
        &mut self,
        ctx: &mut Context<Self::Event>,
        agent_state: &mut Self::AgentState,
    ) -> bool {
        let mut context = Context::from_upper(&mut *ctx);

        // 1. Execute next scheduled behaviour.
        self.inner.scheduler().action(&mut context, agent_state);

        // 2. Handle events the behaviour produced.
        while let Some(event) = context.local.events.pop() {
            self.inner.handle_child_event(event);
        }

        // 3. Update the parent context.
        ctx.merge(context);

        // 4. Run user defined actions for this complex behaviour.
        self.inner.after_child_action(ctx, agent_state);

        self.inner.scheduler().is_finished()
    }

    fn reset(&mut self) {
        self.inner.reset();
    }
}
