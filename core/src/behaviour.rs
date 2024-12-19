use alloc::boxed::Box;

pub use self::complex::{parallel, sequential, ComplexBehaviour};
pub use self::context::Context;
pub use self::simple::{CyclicBehaviour, OneShotBehaviour, TickerBehaviour};

use self::parallel::ParallelBehaviour;
use self::sequential::SequentialBehaviour;
use self::simple::TickerBehaviourWrapper;

pub(crate) mod complex;
mod context;
mod simple;

pub fn oneshot<M: 'static>(
    oneshot: impl OneShotBehaviour<Message = M> + 'static,
) -> Box<dyn Behaviour<Message = M>> {
    Box::new(SimpleBehaviourKind::OneShot(Some(Box::new(oneshot))))
}

pub fn cyclic<M: 'static>(
    cyclic: impl CyclicBehaviour<Message = M> + 'static,
) -> Box<dyn Behaviour<Message = M>> {
    Box::new(SimpleBehaviourKind::Cyclic(Box::new(cyclic)))
}

pub fn ticker<T, M: 'static>(ticker: T) -> Box<dyn Behaviour<Message = M>>
where
    T: TickerBehaviour<Message = M> + 'static,
{
    Box::new(SimpleBehaviourKind::Ticker(TickerBehaviourWrapper::new(
        T::interval(),
        Box::new(ticker),
    )))
}

pub fn sequential<M: 'static, CM: 'static>(
    sequential: impl sequential::SequentialBehaviour<CM, Message = M> + 'static,
) -> Box<dyn Behaviour<Message = M>> {
    Box::new(ComplexBehaviourKind::<_, CM>::Sequential(Box::new(
        sequential,
    )))
}

pub fn parallel<M: 'static, CM: 'static>(
    parallel: impl parallel::ParallelBehaviour<CM, Message = M> + 'static,
) -> Box<dyn Behaviour<Message = M>> {
    Box::new(ComplexBehaviourKind::Parallel(Box::new(parallel)))
}

pub trait Behaviour: 'static {
    type Message;

    fn action(&mut self, ctx: &mut Context<Self::Message>) -> bool;
}

enum SimpleBehaviourKind<M> {
    // Wrapped in an [`Option`] so the user's implementation can use the owned value.
    OneShot(Option<Box<dyn OneShotBehaviour<Message = M>>>),
    Cyclic(Box<dyn CyclicBehaviour<Message = M>>),
    Ticker(TickerBehaviourWrapper<Box<dyn CyclicBehaviour<Message = M>>>),
}

enum ComplexBehaviourKind<M, CM> {
    Sequential(Box<dyn SequentialBehaviour<CM, Message = M>>),
    Parallel(Box<dyn ParallelBehaviour<CM, Message = M>>),
}

impl<M: 'static> Behaviour for SimpleBehaviourKind<M> {
    type Message = M;

    fn action(&mut self, ctx: &mut Context<Self::Message>) -> bool {
        match self {
            SimpleBehaviourKind::OneShot(oneshot) => {
                oneshot
                    .take()
                    .expect("oneshot behaviour should only be called once")
                    .action(ctx);
                true
            }
            SimpleBehaviourKind::Cyclic(cyclic) => {
                cyclic.action(ctx);
                cyclic.is_finished()
            }
            SimpleBehaviourKind::Ticker(ticker) => {
                ticker.action(ctx);
                ticker.is_finished()
            }
        }
    }
}

impl<M: 'static, CM: 'static> Behaviour for ComplexBehaviourKind<M, CM> {
    type Message = M;

    fn action(&mut self, _: &mut Context<Self::Message>) -> bool {
        use self::complex::BehaviourQueue;

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
