use alloc::boxed::Box;
use alloc::vec::Vec;

use log::{Level, log};

use ember_core::agent::Aid;
use ember_core::message::content::ember_bdil::BdilContent;
use ember_core::message::{Content, Message, Performative, Receiver};

use crate::bindings::BindingLookup;
use crate::context::Context;
use crate::event::Trigger;
use crate::literal::Literal;
use crate::resolve::{Resolve, ResolveFailure};
use crate::term::{Structure, Term};
use crate::variable::Variable;

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
    SendLiteral(VariableOrReceiver, Trigger, Literal),
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
            SendLiteral(receiver, trigger, literal) => {
                let literal = Literal {
                    negated: false,
                    structure: Structure {
                        functor: "message".into(),
                        arguments: Some(Box::new([Term::Literal(literal)])),
                    },
                };
                let performative = match trigger {
                    Trigger::Addition => Performative::Inform,
                    Trigger::Deletion => Performative::NotUnderstood,
                };
                let receiver = match receiver.resolve(bindings) {
                    Ok(VariableOrReceiver::Receiver(r)) => r,
                    Ok(_) => {
                        log::error!("failed to resolve .send arguments");
                        return;
                    }
                    Err(_) => {
                        log::error!("failed to parse receiver");
                        return;
                    }
                };
                context.send_message(Message {
                    performative,
                    receiver: Some(receiver),
                    ontology: None,
                    other: None,
                    content: Some(Content::Bdil(BdilContent::Literal(literal.into()))),
                });
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariableOrReceiver {
    Variable(Variable),
    Receiver(Receiver),
}

impl Resolve for VariableOrReceiver {
    type View<'a>
        = Self
    where
        Self: 'a;

    fn resolve(self, bindings: &impl BindingLookup) -> Result<Self, ResolveFailure> {
        self.resolve_as_view(bindings)
    }

    fn resolve_as_view<'a>(
        &'a self,
        bindings: &'a impl BindingLookup,
    ) -> Result<Self::View<'a>, ResolveFailure> {
        Ok(match self {
            VariableOrReceiver::Variable(v) => match bindings.lookup_as_type::<Aid>(v) {
                Some(Ok(aid)) => VariableOrReceiver::Receiver(Receiver::Single(aid)),
                Some(Err(e)) => return Err(ResolveFailure::ConversionFailed(e)),
                None => VariableOrReceiver::Variable(v.clone()),
            },
            VariableOrReceiver::Receiver(r) => self.clone(),
        })
    }
}
