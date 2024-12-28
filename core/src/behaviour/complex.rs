use alloc::boxed::Box;

use super::{Behaviour, Context, IntoBehaviour};

use self::parallel::{ParallelBehaviour, ParallelBehaviourQueue};
use self::sequential::{SequentialBehaviour, SequentialBehaviourQueue};

pub mod parallel;
pub mod sequential;

mod macros;

macro_rules! complex_action_impl {
    () => {
        fn action(&mut self, ctx: &mut Context<Self::Message>) -> bool {
            let mut context = Context::new();

            // 1. Execute next scheduled behaviour.
            self.queue.action(&mut context);

            // 2. Handle messages the behaviour produced.
            if let Some(mut messages) = context.messages.take() {
                while let Some(message) = messages.pop() {
                    self.kind.0.handle_child_message(message);
                }
            }

            // 4. Run user defined actions for this complex behaviour.
            ctx.merge(context);
            self.kind.0.after_child_action(ctx);

            self.queue.is_finished()
        }
    };
}

struct ComplexBehaviour<K, Q> {
    kind: K,
    queue: Q,
}

struct SequentialBehaviourImpl<S: SequentialBehaviour>(S);
struct ParallelBehaviourImpl<P: ParallelBehaviour>(P);

impl<S, M: 'static, CM: 'static> Behaviour
    for ComplexBehaviour<SequentialBehaviourImpl<S>, SequentialBehaviourQueue<CM>>
where
    S: SequentialBehaviour<Message = M, ChildMessage = CM> + 'static,
{
    type Message = M;

    complex_action_impl!();
}

impl<P, M: 'static, CM: 'static> Behaviour
    for ComplexBehaviour<ParallelBehaviourImpl<P>, ParallelBehaviourQueue<CM>>
where
    P: ParallelBehaviour<Message = M, ChildMessage = CM> + 'static,
{
    type Message = M;

    complex_action_impl!();
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
