use self::scheduler::BehaviourScheduler;
use super::{get_id, Behaviour, BehaviourId, Context, IntoBehaviour};

pub mod fsm;
pub mod parallel;
pub mod sequential;

mod blocked;
pub(crate) mod scheduler;

pub trait ComplexBehaviour {
    type Event;

    type ChildEvent;

    fn handle_child_event(&mut self, message: Self::ChildEvent) {
        let _ = message;
    }

    fn after_child_action(&mut self, ctx: &mut Context<Self::Event>) {
        let _ = ctx;
    }
}

trait ScheduledComplexBehaviour: ComplexBehaviour
where
    Self::ChildEvent: 'static,
{
    fn scheduler(&mut self) -> &mut impl BehaviourScheduler<Self::ChildEvent>;
}

struct ComplexBehaviourImpl<I> {
    id: BehaviourId,
    inner: I,
}

impl<I, E: 'static, CE: 'static> Behaviour for ComplexBehaviourImpl<I>
where
    I: ScheduledComplexBehaviour<Event = E, ChildEvent = CE> + 'static,
{
    type Event = E;

    fn id(&self) -> BehaviourId {
        self.id
    }

    fn action(&mut self, ctx: &mut Context<Self::Event>) -> bool {
        let mut context = Context::from_upper(&mut *ctx);

        // 1. Execute next scheduled behaviour.
        self.inner.scheduler().action(&mut context);

        // 2. Handle events the behaviour produced.
        while let Some(event) = context.local.events.pop() {
            self.inner.handle_child_event(event);
        }

        // 3. Update the parent context.
        ctx.merge(context);

        // 4. Run user defined actions for this complex behaviour.
        self.inner.after_child_action(ctx);

        self.inner.scheduler().is_finished()
    }
}
