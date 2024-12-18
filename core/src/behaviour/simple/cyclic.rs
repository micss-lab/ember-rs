use super::Context;

pub trait CyclicBehaviour {
    type Message;

    fn action(&mut self, ctx: &mut Context<Self::Message>);

    fn is_finished(&self) -> bool;
}
