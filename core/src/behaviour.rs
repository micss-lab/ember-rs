use alloc::boxed::Box;

pub use self::complex::{parallel, sequential, ComplexBehaviour};
pub use self::context::Context;
pub use self::simple::{CyclicBehaviour, OneShotBehaviour, TickerBehaviour};

use self::simple::TickerBehaviourWrapper;

pub(crate) mod complex;
mod context;
mod kind;
mod simple;

pub trait Behaviour: 'static {
    type Message;

    fn action(&mut self, ctx: &mut Context<Self::Message>) -> bool;
}

pub fn oneshot<M: 'static>(
    oneshot: impl OneShotBehaviour<Message = M> + 'static,
) -> Box<dyn Behaviour<Message = M>> {
    Box::new(kind::SimpleBehaviour::OneShot(Some(Box::new(oneshot))))
}

pub fn cyclic<M: 'static>(
    cyclic: impl CyclicBehaviour<Message = M> + 'static,
) -> Box<dyn Behaviour<Message = M>> {
    Box::new(kind::SimpleBehaviour::Cyclic(Box::new(cyclic)))
}

pub fn ticker<T, M: 'static>(ticker: T) -> Box<dyn Behaviour<Message = M>>
where
    T: TickerBehaviour<Message = M> + 'static,
{
    Box::new(kind::SimpleBehaviour::Ticker(TickerBehaviourWrapper::new(
        T::interval(),
        Box::new(ticker),
    )))
}

pub fn sequential<M: 'static, CM: 'static>(
    sequential: impl sequential::SequentialBehaviour<CM, Message = M> + 'static,
) -> Box<dyn Behaviour<Message = M>> {
    Box::new(kind::ComplexBehaviour::<_, CM>::Sequential(Box::new(
        sequential,
    )))
}

pub fn parallel<M: 'static, CM: 'static>(
    parallel: impl parallel::ParallelBehaviour<CM, Message = M> + 'static,
) -> Box<dyn Behaviour<Message = M>> {
    Box::new(kind::ComplexBehaviour::Parallel(Box::new(parallel)))
}
