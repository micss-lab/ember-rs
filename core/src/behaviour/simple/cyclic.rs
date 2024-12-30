use alloc::boxed::Box;

use super::{Behaviour, Context, IntoBehaviour};

pub trait CyclicBehaviour {
    type Message;

    fn action(&mut self, ctx: &mut Context<Self::Message>);

    fn is_finished(&self) -> bool;
}

struct CyclicBehaviourImpl<C: CyclicBehaviour>(C);

impl<M: 'static, C> Behaviour for CyclicBehaviourImpl<C>
where
    C: CyclicBehaviour<Message = M> + 'static,
{
    type Message = M;

    fn action(&mut self, ctx: &mut Context<Self::Message>) -> bool {
        self.0.action(ctx);
        self.0.is_finished()
    }
}

#[doc(hidden)]
pub struct Cyclic;

impl<T, M: 'static> IntoBehaviour<Cyclic> for T
where
    T: CyclicBehaviour<Message = M> + 'static,
{
    type Message = M;

    fn into_behaviour(self) -> Box<dyn Behaviour<Message = Self::Message>> {
        Box::new(CyclicBehaviourImpl(self))
    }
}
