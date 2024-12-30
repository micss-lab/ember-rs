use alloc::boxed::Box;

use super::{get_id, Behaviour, BehaviourId, Context, IntoBehaviour};

pub trait OneShotBehaviour {
    type Message;

    fn action(&self, ctx: &mut Context<Self::Message>);
}

struct OneShotBehaviourImpl<O: OneShotBehaviour> {
    id: BehaviourId,
    oneshot: Option<O>,
}

impl<M: 'static, O> Behaviour for OneShotBehaviourImpl<O>
where
    O: OneShotBehaviour<Message = M> + 'static,
{
    type Message = M;

    fn id(&self) -> BehaviourId {
        self.id
    }

    fn action(&mut self, ctx: &mut Context<Self::Message>) -> bool {
        self.oneshot
            .take()
            .expect("oneshot behaviour should only be called once")
            .action(ctx);
        true
    }
}

#[doc(hidden)]
pub struct OneShot;

impl<T, M: 'static> IntoBehaviour<OneShot> for T
where
    T: OneShotBehaviour<Message = M> + 'static,
{
    type Message = M;

    fn into_behaviour(self) -> Box<dyn Behaviour<Message = Self::Message>> {
        Box::new(OneShotBehaviourImpl {
            id: get_id(),
            oneshot: Some(self),
        })
    }
}
