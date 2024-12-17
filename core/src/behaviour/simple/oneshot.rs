use super::Context;

pub trait OneShotBehaviour {
    fn action(&self, ctx: &mut Context);
}
