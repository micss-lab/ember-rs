use alloc::boxed::Box;
use alloc::vec::Vec;

pub use self::complex::{parallel, sequential};
pub use self::simple::{CyclicBehaviour, OneShotBehaviour, TickerBehaviour};

pub use crate::context::Context;

pub(crate) mod complex;
mod simple;

pub(crate) type BehaviourVec<M> = Vec<Box<dyn Behaviour<Message = M>>>;

pub trait Behaviour: 'static {
    type Message;

    fn action(&mut self, ctx: &mut Context<Self::Message>) -> bool;
}

pub trait IntoBehaviour<Kind>
where
    Self: Sized,
{
    type Message;

    fn into_behaviour(self) -> Box<dyn Behaviour<Message = Self::Message>>;
}
