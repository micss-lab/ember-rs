use super::Context;

pub trait CyclicBehaviour {
    fn action(&mut self, ctx: &mut Context);

    fn is_finished(&self) -> bool;
}
