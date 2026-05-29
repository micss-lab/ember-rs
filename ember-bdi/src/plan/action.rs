use crate::context::Context;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action<A> {
    System(SystemAction),
    User(A),
}

#[derive(derive_more::Debug, Clone, PartialEq, Eq)]
pub enum SystemAction {}

impl SystemAction {
    pub(crate) fn execute<A>(self, _context: &mut Context<A>) {
        match self {}
    }
}
