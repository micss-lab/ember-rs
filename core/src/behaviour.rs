pub use self::context::Context;

pub use self::complex::ComplexBehaviour;
pub use self::complex::{parallel, sequential};
pub use self::complex::{ParallelBehaviour, ParallelBehaviourQueue, ParallelFinishStrategy};
pub use self::complex::{SequentialBehaviour, SequentialBehaviourQueue};

pub use self::simple::{cyclic, oneshot, ticker};
pub use self::simple::{CyclicBehaviour, OneShotBehaviour, TickerBehaviour};

pub(crate) mod complex;
mod context;
mod simple;

pub trait Behaviour: 'static {
    type Message;

    fn action(&mut self, ctx: &mut Context<Self::Message>) -> bool;
}
