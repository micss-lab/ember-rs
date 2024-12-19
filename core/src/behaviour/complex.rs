use alloc::boxed::Box;

use super::{Behaviour, Context, IntoBehaviour};

use self::parallel::ParallelBehaviour;
use self::sequential::SequentialBehaviour;

pub mod parallel;
pub mod sequential;

pub trait ComplexBehaviour<M, Ord>
where
    Self: Sized,
{
    fn add_behaviour<K>(&mut self, behaviour: impl IntoBehaviour<K, Message = M>);

    fn with_behaviour<K>(mut self, behaviour: impl IntoBehaviour<K, Message = M>) -> Self {
        self.add_behaviour(behaviour);
        self
    }
}

enum ComplexBehaviourKind<M, CM> {
    Sequential(Box<dyn SequentialBehaviour<ChildMessage = CM, Message = M>>),
    Parallel(Box<dyn ParallelBehaviour<ChildMessage = CM, Message = M>>),
}

impl<M: 'static, CM: 'static> Behaviour for ComplexBehaviourKind<M, CM> {
    type Message = M;

    fn action(&mut self, _: &mut Context<Self::Message>) -> bool {
        let mut context = Context::new();
        let mut queue_action = |queue: &mut dyn BehaviourQueue<CM>| {
            queue.action(&mut context);
            queue.is_finished()
        };

        match self {
            ComplexBehaviourKind::Sequential(sequential) => queue_action(sequential.queue()),
            ComplexBehaviourKind::Parallel(parallel) => queue_action(parallel.queue()),
        }
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
