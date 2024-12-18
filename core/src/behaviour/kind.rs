use alloc::boxed::Box;

use super::{
    parallel::ParallelBehaviour, sequential::SequentialBehaviour, Behaviour, Context,
    CyclicBehaviour, OneShotBehaviour, TickerBehaviourWrapper,
};

/// Implemented for types that represent a behaviour.
// pub(super) enum BehaviourKind<M> {
//     OneShot(Option<Box<dyn OneShotBehaviour>>),
//     Cyclic(Box<dyn CyclicBehaviour>),
//     Ticker(TickerBehaviourWrapper<Box<dyn CyclicBehaviour>>),
//     Sequential(Box<dyn SequentialBehaviour>),
//     Parallel(Box<dyn ParallelBehaviour<Message = M>>),
// }

pub(super) enum SimpleBehaviour<M> {
    OneShot(Option<Box<dyn OneShotBehaviour<Message = M>>>),
    Cyclic(Box<dyn CyclicBehaviour<Message = M>>),
    Ticker(TickerBehaviourWrapper<Box<dyn CyclicBehaviour<Message = M>>>),
}

pub(super) enum ComplexBehaviour<M, CM> {
    Sequential(Box<dyn SequentialBehaviour<CM, Message = M>>),
    Parallel(Box<dyn ParallelBehaviour<CM, Message = M>>),
}

impl<M: 'static> Behaviour for SimpleBehaviour<M> {
    type Message = M;

    fn action(&mut self, ctx: &mut Context<Self::Message>) -> bool {
        match self {
            SimpleBehaviour::OneShot(oneshot) => {
                oneshot.take().unwrap().action(ctx);
                true
            }
            SimpleBehaviour::Cyclic(_) => todo!(),
            SimpleBehaviour::Ticker(_) => todo!(),
        }
    }
}

impl<M: 'static, CM: 'static> Behaviour for ComplexBehaviour<M, CM> {
    type Message = M;

    fn action(&mut self, _: &mut Context<Self::Message>) -> bool {
        use super::complex::BehaviourQueue;

        let mut context = Context::new();

        match self {
            ComplexBehaviour::Sequential(_) => todo!(),
            ComplexBehaviour::Parallel(parallel) => {
                parallel.queue().action(&mut context);
                parallel.queue().is_finished()
            }
        }
    }
}
