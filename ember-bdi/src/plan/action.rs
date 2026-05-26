use alloc::rc::Rc;

use crate::context::Context;

#[derive(Debug, Clone)]
pub enum Action<A> {
    System(SystemAction<A>),
    User(A),
}

#[derive(derive_more::Debug, Clone)]
pub enum SystemAction<A> {
    // TODO: Remove this action.
    Boxed(#[debug("boxed action")] Rc<dyn Fn(&mut Context<A>)>),
}

impl<A> SystemAction<A> {
    pub(crate) fn execute(self, context: &mut Context<A>) {
        match self {
            SystemAction::Boxed(f) => f(context),
        }
    }
}
