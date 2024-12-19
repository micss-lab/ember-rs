use alloc::boxed::Box;

use super::{Behaviour, Context, IntoBehaviour, SimpleBehaviourKind};

pub trait OneShotBehaviour {
    type Message;

    fn action(&self, ctx: &mut Context<Self::Message>);
}

#[doc(hidden)]
pub struct OneShot;

impl<T, M: 'static> IntoBehaviour<OneShot> for T
where
    T: OneShotBehaviour<Message = M> + 'static,
{
    type Message = M;

    fn into_behaviour(self) -> Box<dyn Behaviour<Message = Self::Message>> {
        Box::new(SimpleBehaviourKind::OneShot(Some(Box::new(self))))
    }
}
