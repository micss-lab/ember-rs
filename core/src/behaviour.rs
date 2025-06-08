use alloc::boxed::Box;
use alloc::vec::Vec;

pub use self::complex::{parallel, sequential};
pub use self::simple::{CyclicBehaviour, OneShotBehaviour, TickerBehaviour};

pub use crate::context::Context;

use crate::util::sync::AtomicU32;

pub(crate) mod complex;
mod simple;

pub(crate) type BehaviourVec<E> = Vec<Box<dyn Behaviour<Event = E>>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct BehaviourId(u32);

pub trait Behaviour: 'static {
    type Event;

    fn id(&self) -> BehaviourId;

    fn action(&mut self, ctx: &mut Context<Self::Event>) -> bool;
}

pub trait IntoBehaviour<Kind>
where
    Self: Sized,
{
    type Event;

    fn into_behaviour(self) -> Box<dyn Behaviour<Event = Self::Event>>;
}

// This way the user can convert the behaviour to a boxed one by themselves and still pass it to
// functions expecting and "IntoBehaviour" impl.
struct BoxedBehviour;
impl<E> IntoBehaviour<BoxedBehviour> for Box<dyn Behaviour<Event = E>> {
    type Event = E;

    fn into_behaviour(self) -> Box<dyn Behaviour<Event = Self::Event>> {
        self
    }
}

fn get_id() -> BehaviourId {
    static ID_COUNTER: AtomicU32 = AtomicU32::new(0);
    BehaviourId(ID_COUNTER.get_increment())
}
