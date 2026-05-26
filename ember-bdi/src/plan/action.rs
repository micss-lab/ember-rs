use alloc::boxed::Box;

use derive_more::Debug;

use crate::context::Context;

#[derive(Debug)]
pub enum Action<A> {
    System(SystemAction<A>),
    User(A),
}

#[derive(Debug)]
pub enum SystemAction<A> {
    Boxed(#[debug("boxed action")] Box<dyn FnOnce(&mut Context<A>)>),
}

impl<A> SystemAction<A> {
    pub(crate) fn execute(self, context: &mut Context<A>) {
        match self {
            SystemAction::Boxed(f) => f(context),
        }
    }
}
