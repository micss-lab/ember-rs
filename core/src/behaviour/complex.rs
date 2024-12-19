use alloc::boxed::Box;

use super::{Behaviour, Context, IntoBehaviour};

use self::parallel::ParallelBehaviour;
use self::sequential::SequentialBehaviour;

pub mod parallel;
pub mod sequential;

struct ComplexBehaviour<Q, M, MC> {
    kind: ComplexBehaviourKind<M, MC>,
    queue: Q,
}

enum ComplexBehaviourKind<M, CM> {
    Sequential(Box<dyn SequentialBehaviour<ChildMessage = CM, Message = M>>),
    Parallel(Box<dyn ParallelBehaviour<ChildMessage = CM, Message = M>>),
}

impl<Q, M: 'static, CM: 'static> Behaviour for ComplexBehaviour<Q, M, CM>
where
    Q: BehaviourQueue<CM> + 'static,
{
    type Message = M;

    fn action(&mut self, ctx: &mut Context<Self::Message>) -> bool {
        let mut context = Context::new();
        self.queue.action(&mut context);
        match &mut self.kind {
            ComplexBehaviourKind::Sequential(sequential) => sequential.after_child_action(ctx),
            ComplexBehaviourKind::Parallel(parallel) => parallel.after_child_action(ctx),
        }
        self.queue.is_finished()
    }
}

pub(crate) trait BehaviourQueue<M: 'static> {
    fn next(&mut self) -> Option<Box<dyn Behaviour<Message = M>>>;

    fn schedule(&mut self, behaviour: Box<dyn Behaviour<Message = M>>);

    fn is_finished(&self) -> bool;

    fn action(&mut self, ctx: &mut Context<M>) -> bool {
        let Some(mut behaviour) = self.next() else {
            return self.is_finished();
        };
        let finished = behaviour.action(ctx);
        if !finished {
            self.schedule(behaviour);
        }
        self.is_finished()
    }
}
