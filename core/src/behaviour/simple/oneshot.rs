use alloc::boxed::Box;

use super::{get_id, Behaviour, BehaviourId, Context, IntoBehaviour};

pub trait OneShotBehaviour {
    type Event;

    fn action(&self, ctx: &mut Context<Self::Event>);

    fn reset(&mut self) {}
}

struct OneShotBehaviourImpl<O: OneShotBehaviour> {
    id: BehaviourId,
    oneshot: O,
}

impl<E: 'static, O> Behaviour for OneShotBehaviourImpl<O>
where
    O: OneShotBehaviour<Event = E> + 'static,
{
    type Event = E;

    fn id(&self) -> BehaviourId {
        self.id
    }

    fn action(&mut self, ctx: &mut Context<Self::Event>) -> bool {
        self.oneshot.action(ctx);
        true
    }

    fn reset(&mut self) {
        self.oneshot.reset();
    }
}

#[doc(hidden)]
pub struct OneShot;

impl<T, E: 'static> IntoBehaviour<OneShot> for T
where
    T: OneShotBehaviour<Event = E> + 'static,
{
    type Event = E;

    fn into_behaviour(self) -> Box<dyn Behaviour<Event = Self::Event>> {
        Box::new(OneShotBehaviourImpl {
            id: get_id(),
            oneshot: self,
        })
    }
}
