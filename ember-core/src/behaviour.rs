use alloc::boxed::Box;

use crate::context::Context;
use crate::util::sync::AtomicU32;

pub mod complex;
pub mod simple;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct BehaviourId(u32);

pub trait Behaviour: 'static {
    type Event;

    type AgentState;

    fn id(&self) -> BehaviourId;

    fn action(&mut self, ctx: &mut Context<Self::Event>, state: &mut Self::AgentState) -> bool;

    fn reset(&mut self);
}

pub trait IntoBehaviour<Kind>
where
    Self: Sized,
{
    type Event;

    type AgentState;

    fn into_behaviour(
        self,
    ) -> Box<dyn Behaviour<AgentState = Self::AgentState, Event = Self::Event>>;
}

// This way the user can convert the behaviour to a boxed one by themselves and still pass it to
// functions expecting and "IntoBehaviour" impl.
#[doc(hidden)]
pub struct BoxedBehviour;
impl<S, E> IntoBehaviour<BoxedBehviour> for Box<dyn Behaviour<AgentState = S, Event = E>> {
    type Event = E;

    type AgentState = S;

    fn into_behaviour(
        self,
    ) -> Box<dyn Behaviour<AgentState = Self::AgentState, Event = Self::Event>> {
        self
    }
}

fn get_id() -> BehaviourId {
    static ID_COUNTER: AtomicU32 = AtomicU32::new(0);
    BehaviourId(ID_COUNTER.get_increment())
}
