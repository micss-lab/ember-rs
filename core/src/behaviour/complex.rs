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

struct ComplexBehaviourImpl<I, S> {
    id: BehaviourId,
    user_impl: I,
    scheduler: S,
}

impl<I, S, E: 'static, CE: 'static> Behaviour for ComplexBehaviourImpl<I, S>
where
    I: ComplexBehaviour<Event = E, ChildEvent = CE> + 'static,
    S: BehaviourScheduler<CE> + 'static,
{
    type Event = E;

    fn id(&self) -> BehaviourId {
        self.id
    }

    fn action(&mut self, ctx: &mut Context<Self::Event>) -> bool {
        let mut context = Context::from_upper(&mut *ctx);

        // 1. Execute next scheduled behaviour.
        self.scheduler.action(&mut context);

        // 2. Handle events the behaviour produced.
        while let Some(event) = context.local.events.pop() {
            self.user_impl.handle_child_event(event);
        }

        // 3. Update the parent context.
        ctx.merge(context);

        // 4. Run user defined actions for this complex behaviour.
        self.user_impl.after_child_action(ctx);

        self.scheduler.is_finished()
    }
}
