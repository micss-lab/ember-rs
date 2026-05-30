use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::vec::Vec;

use log::{Level, log};

use crate::bindings::BindingLookup;
use crate::bindings::resolver::Resolve;
use crate::context::Context;
use crate::term::Term;

pub trait Execute: Sized {
    type Agent;
    /// The action stored in the context. In almost all cases, this can just be `Self`.
    type Action;

    fn execute(
        self,
        bindings: &impl BindingLookup,
        context: &mut Context<Self::Action>,
        agent: &mut Self::Agent,
    );
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action<A> {
    Builtin(BuiltinAction),
    User(A),
}

impl<Agent, A> Execute for Action<A>
where
    A: Execute<Agent = Agent, Action = A>,
{
    type Agent = Agent;
    type Action = A;

    fn execute(
        self,
        bindings: &impl BindingLookup,
        context: &mut Context<Self::Action>,
        agent: &mut Self::Agent,
    ) {
        match self {
            Action::Builtin(action) => action.execute(bindings, context),
            Action::User(action) => action.execute(bindings, context, agent),
        }
    }
}

#[derive(derive_more::Debug, Clone, PartialEq, Eq)]
pub enum BuiltinAction {
    Log(Level, Cow<'static, str>, Option<Box<[Term]>>),
}

impl BuiltinAction {
    pub(crate) fn execute<A>(self, bindings: &impl BindingLookup, _context: &mut Context<A>) {
        match self {
            BuiltinAction::Log(level, text, Some(terms)) => {
                let terms = terms
                    .into_iter()
                    .map(|t| t.resolve(bindings))
                    .collect::<Result<Vec<_>, _>>()
                    .expect("failed to resolve log arguments");
                log!(level, "{} {:?}", text, terms)
            }
            BuiltinAction::Log(level, text, None) => {
                log!(level, "{}", text)
            }
        }
    }
}
