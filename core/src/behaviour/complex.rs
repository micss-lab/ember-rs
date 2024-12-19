use alloc::boxed::Box;

use super::{Behaviour, Context};

pub use self::parallel::{
    FinishStrategy as ParallelFinishStrategy, ParallelBehaviour, ParallelBehaviourQueue,
};
pub use self::sequential::{SequentialBehaviour, SequentialBehaviourQueue};

pub(crate) mod parallel;
mod sequential;

pub trait ComplexBehaviour<M, Ord>
where
    Self: Sized,
{
    fn add_behaviour(&mut self, behaviour: impl Behaviour<Message = M>);

    fn with_behaviour(mut self, behaviour: impl Behaviour<Message = M>) -> Self {
        self.add_behaviour(behaviour);
        self
    }
}

pub fn sequential<S, M: 'static, CM: 'static>(sequential: S) -> Box<dyn Behaviour<Message = M>>
where
    S: SequentialBehaviour<ChildMessage = CM, Message = M> + 'static,
{
    Box::new(ComplexBehaviourKind::<_, CM>::Sequential(Box::new(
        sequential,
    )))
}

pub fn parallel<P, M: 'static, CM: 'static>(parallel: P) -> Box<dyn Behaviour<Message = M>>
where
    P: ParallelBehaviour<ChildMessage = CM, Message = M> + 'static,
{
    Box::new(ComplexBehaviourKind::Parallel(Box::new(parallel)))
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
