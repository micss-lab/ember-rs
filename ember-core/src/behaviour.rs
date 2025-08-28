pub use self::complex::ComplexBehaviour;
pub use self::complex::{fsm, parallel, sequential};
pub use self::simple::cyclic::CyclicBehaviour;
pub use self::simple::oneshot::OneShotBehaviour;
pub use self::simple::ticker::TickerBehaviour;
pub use crate::context::Context;

use alloc::boxed::Box;

use crate::util::sync::AtomicU32;

pub mod complex;
pub mod simple;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct BehaviourId(u32);

pub trait Behaviour {
    type Event;

    type AgentState;

    fn id(&self) -> BehaviourId;

    fn action(&mut self, ctx: &mut Context<Self::Event>, state: &mut Self::AgentState) -> bool;

    fn reset(&mut self);
}

pub trait IntoBehaviour<'a, Kind>
where
    Self: Sized,
{
    type Event;

    type AgentState;

    fn into_behaviour(
        self,
    ) -> Box<dyn Behaviour<AgentState = Self::AgentState, Event = Self::Event> + 'a>;
}

pub trait IntoBehaviourWithId<'a, Kind>: IntoBehaviour<'a, Kind>
where
    Self: Sized,
{
    fn into_behaviour_with_id(
        self,
    ) -> (
        BehaviourId,
        Box<dyn Behaviour<AgentState = Self::AgentState, Event = Self::Event> + 'a>,
    ) {
        let behaviour = self.into_behaviour();
        (behaviour.id(), behaviour)
    }
}

impl<'a, K, B> IntoBehaviourWithId<'a, K> for B where B: IntoBehaviour<'a, K> {}

// This way the user can convert the behaviour to a boxed one by themselves and still pass it to
// functions expecting and "IntoBehaviour" impl.
#[doc(hidden)]
pub struct BoxedBehviour;
impl<'a, S, E> IntoBehaviour<'a, BoxedBehviour>
    for Box<dyn Behaviour<AgentState = S, Event = E> + 'a>
{
    type Event = E;

    type AgentState = S;

    fn into_behaviour(
        self,
    ) -> Box<dyn Behaviour<AgentState = Self::AgentState, Event = Self::Event> + 'a> {
        self
    }
}

fn get_id() -> BehaviourId {
    static ID_COUNTER: AtomicU32 = AtomicU32::new(0);
    BehaviourId(ID_COUNTER.get_increment())
}
