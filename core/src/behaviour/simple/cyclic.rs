use alloc::boxed::Box;

use super::{get_id, Behaviour, BehaviourId, Context, IntoBehaviour};

pub trait CyclicBehaviour {
    type Event;

    fn action(&mut self, ctx: &mut Context<Self::Event>);

    fn is_finished(&self) -> bool;

    fn reset(&mut self) {}
}

struct CyclicBehaviourImpl<C: CyclicBehaviour> {
    id: BehaviourId,
    cyclic: C,
}

impl<E: 'static, C> Behaviour for CyclicBehaviourImpl<C>
where
    C: CyclicBehaviour<Event = E> + 'static,
{
    type Event = E;

    fn id(&self) -> BehaviourId {
        self.id
    }

    fn action(&mut self, ctx: &mut Context<Self::Event>) -> bool {
        self.cyclic.action(ctx);
        self.cyclic.is_finished()
    }

    fn reset(&mut self) {
        self.cyclic.reset();
    }
}

#[doc(hidden)]
pub struct Cyclic;

impl<T, E: 'static> IntoBehaviour<Cyclic> for T
where
    T: CyclicBehaviour<Event = E> + 'static,
{
    type Event = E;

    fn into_behaviour(self) -> Box<dyn Behaviour<Event = Self::Event>> {
        Box::new(CyclicBehaviourImpl {
            id: get_id(),
            cyclic: self,
        })
    }
}
