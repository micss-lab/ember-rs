use alloc::boxed::Box;
use alloc::vec::Vec;

use log::{Level, log};

use crate::bindings::BindingLookup;
use crate::context::Context;
use crate::resolve::Resolve;
use crate::term::Term;

pub trait Execute: Sized {
    type State;
    /// The action stored in the context. In almost all cases, this can just be `Self`.
    type Action;

    fn execute(
        self,
        bindings: &impl BindingLookup,
        context: &mut Context<Self::Action>,
        state: &mut Self::State,
    );
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action<A> {
    Builtin(BuiltinAction),
    User(A),
}

impl<State, A> Execute for Action<A>
where
    A: Execute<State = State, Action = A>,
{
    type State = State;
    type Action = A;

    fn execute(
        self,
        bindings: &impl BindingLookup,
        context: &mut Context<Self::Action>,
        state: &mut Self::State,
    ) {
        match self {
            Action::Builtin(action) => action.execute(bindings, context),
            Action::User(action) => action.execute(bindings, context, state),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuiltinAction {
    Log(Level, Box<[Term]>),
    StopPlatform,
}

impl BuiltinAction {
    pub(crate) fn execute<A>(self, bindings: &impl BindingLookup, context: &mut Context<A>) {
        use BuiltinAction::*;
        match self {
            Log(level, terms) => {
                match terms
                    .into_iter()
                    .map(|t| t.resolve(bindings))
                    .collect::<Result<Vec<_>, _>>()
                {
                    Ok(terms) => log!(level, "{terms:?}"),
                    Err(_) => log::error!("failed to resolve log arguments"),
                }
            }
            StopPlatform => context.stop_platform(),
        }
    }
}
