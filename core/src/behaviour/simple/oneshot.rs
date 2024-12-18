use super::Context;

pub trait OneShotBehaviour {
    type Message;

    fn action(&self, ctx: &mut Context<Self::Message>);
}
