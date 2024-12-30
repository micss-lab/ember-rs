use alloc::boxed::Box;

use super::{Behaviour, Context, IntoBehaviour};

pub mod parallel;
pub mod sequential;

mod macros;

struct ComplexBehaviour<K, Q> {
    kind: K,
    queue: Q,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ScheduleStrategy {
    Next,
    End,
}

pub(crate) trait BehaviourQueue<M: 'static> {
    fn next(&mut self) -> Option<Box<dyn Behaviour<Message = M>>>;

    fn schedule(&mut self, behaviour: Box<dyn Behaviour<Message = M>>, strategy: ScheduleStrategy);

    fn reschedule(&mut self, behaviour: Box<dyn Behaviour<Message = M>>);

    fn is_finished(&self) -> bool;

    fn action(&mut self, ctx: &mut Context<M>) -> bool {
        let Some(mut behaviour) = self.next() else {
            return self.is_finished();
        };
        let finished = behaviour.action(&mut *ctx);

        // Immediatly schedule newly created behaviours.
        if let Some(new_behaviours) = ctx.new_behaviours.take() {
            new_behaviours
                .into_iter()
                .flat_map(|(strategy, behaviours)| {
                    behaviours.into_iter().zip(core::iter::repeat(strategy))
                })
                .for_each(|(behaviour, strategy)| self.schedule(behaviour, strategy));
        }

        if !finished {
            self.reschedule(behaviour);
        }
        self.is_finished()
    }
}
