use alloc::boxed::Box;

use super::{Behaviour, Context, IntoBehaviour, SimpleBehaviourKind};

pub trait CyclicBehaviour {
    type Message;

    fn action(&mut self, ctx: &mut Context<Self::Message>);

    fn is_finished(&self) -> bool;
}

#[doc(hidden)]
pub struct Cyclic;

impl<T, M: 'static> IntoBehaviour<Cyclic> for T
where
    T: CyclicBehaviour<Message = M> + 'static,
{
    type Message = M;

    fn into_behaviour(self) -> Box<dyn Behaviour<Message = Self::Message>> {
        Box::new(SimpleBehaviourKind::Cyclic(Box::new(self)))
    }
}
