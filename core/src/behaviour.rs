use alloc::boxed::Box;

pub use self::complex::{parallel, sequential, ComplexBehaviour};
pub use self::context::Context;
pub use self::simple::{CyclicBehaviour, OneShotBehaviour, TickerBehaviour};

use self::complex::parallel::ParallelBehaviour;
use self::complex::sequential::SequentialBehaviour;
use self::simple::TickerBehaviourWrapper;

pub(crate) mod complex;
mod context;
mod simple;

pub struct Behaviour(BehaviourKind);

/// Implemented for types that represent a behaviour.
enum BehaviourKind {
    OneShot(Option<Box<dyn OneShotBehaviour>>),
    Cyclic(Box<dyn CyclicBehaviour>),
    Ticker(TickerBehaviourWrapper<Box<dyn CyclicBehaviour>>),
    Sequential(SequentialBehaviour),
    Parallel(ParallelBehaviour),
}

impl Behaviour {
    pub(crate) fn action(&mut self, ctx: &mut Context) -> bool {
        match &mut self.0 {
            BehaviourKind::OneShot(oneshot) => {
                let oneshot = oneshot
                    .take()
                    .expect("cannot run oneshot behaviour more than once");
                oneshot.action(ctx);
                true
            }
            BehaviourKind::Cyclic(cyclic) => {
                cyclic.action(ctx);
                cyclic.is_finished()
            }
            BehaviourKind::Ticker(ticker) => ticker.action(ctx),
            BehaviourKind::Sequential(sequential) => {
                use self::complex::BehaviourQueue;
                sequential.action(ctx)
            }
            BehaviourKind::Parallel(parallel) => {
                use self::complex::BehaviourQueue;
                parallel.action(ctx)
            }
        }
    }
}

impl Behaviour {
    pub fn oneshot(oneshot: impl OneShotBehaviour + 'static) -> Behaviour {
        BehaviourKind::OneShot(Some(Box::new(oneshot))).into()
    }

    pub fn cyclic(cyclic: impl CyclicBehaviour + 'static) -> Behaviour {
        BehaviourKind::Cyclic(Box::new(cyclic)).into()
    }

    pub fn ticker<T>(ticker: T) -> Behaviour
    where
        T: TickerBehaviour + 'static,
    {
        BehaviourKind::Ticker(TickerBehaviourWrapper::new(T::interval(), Box::new(ticker))).into()
    }

    pub fn sequential(sequential: SequentialBehaviour) -> Behaviour {
        BehaviourKind::Sequential(sequential).into()
    }

    pub fn parallel(parallel: ParallelBehaviour) -> Behaviour {
        BehaviourKind::Parallel(parallel).into()
    }
}

impl From<BehaviourKind> for Behaviour {
    fn from(value: BehaviourKind) -> Self {
        Behaviour(value)
    }
}
