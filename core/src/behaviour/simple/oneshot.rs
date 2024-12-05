use alloc::boxed::Box;

use super::{Behaviour, Context};

#[allow(clippy::type_complexity)]
pub struct OneShotBehaviour<S>(Box<dyn FnMut(&mut Context, S) -> S + Send>);

impl<S> OneShotBehaviour<S> {
    pub fn new(action: impl FnMut(&mut Context, S) -> S + Send + 'static) -> Self {
        Self(Box::new(action))
    }
}

impl<S> Behaviour for OneShotBehaviour<S> {
    type ParentState = S;

    fn action(&mut self, ctx: &mut Context, state: Self::ParentState) -> (bool, Self::ParentState) {
        (true, self.0(ctx, state))
    }
}
