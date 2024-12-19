use super::{Context, CyclicBehaviour};

pub trait TickerBehaviour {
    type Message;

    fn interval() -> core::time::Duration;

    fn action(&mut self, ctx: &mut Context<Self::Message>);

    fn is_finished(&self) -> bool;
}

impl<T, M> CyclicBehaviour for T
where
    T: TickerBehaviour<Message = M>,
{
    type Message = M;

    fn action(&mut self, ctx: &mut Context<Self::Message>) {
        self.action(ctx)
    }

    fn is_finished(&self) -> bool {
        self.is_finished()
    }
}
