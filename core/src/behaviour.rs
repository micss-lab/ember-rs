use alloc::boxed::Box;
use alloc::vec::Vec;

pub use self::complex::{parallel, sequential};
pub use self::simple::{CyclicBehaviour, OneShotBehaviour, TickerBehaviour};

pub use crate::context::Context;

use crate::util::sync::AtomicU32;

pub(crate) mod complex;
mod simple;

pub(crate) type BehaviourVec<M> = Vec<Box<dyn Behaviour<Message = M>>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct BehaviourId(u32);

pub trait Behaviour: 'static {
    type Message;

    fn id(&self) -> BehaviourId;

    fn action(&mut self, ctx: &mut Context<Self::Message>) -> bool;
}

pub trait IntoBehaviour<Kind>
where
    Self: Sized,
{
    type Message;

    fn into_behaviour(self) -> Box<dyn Behaviour<Message = Self::Message>>;
}

fn get_id() -> BehaviourId {
    static ID_COUNTER: AtomicU32 = AtomicU32::new(0);
    BehaviourId(ID_COUNTER.get_increment())
}
