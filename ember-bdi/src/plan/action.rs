use alloc::boxed::Box;
use alloc::vec::Vec;

use log::{Level, log};

use ember_core::agent::Aid;
use ember_core::message::content::ember_bdil::BdilContent;
use ember_core::message::{Content, Message, Performative, Receiver};

use crate::bindings::{BindingLookup, OwnedBindings};
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

    /// Executes the action returning `None` if it has finshed and a new action state if the action
    /// is to be ran again.
    fn execute(
        self,
        bindings: &impl BindingLookup,
        context: &mut Context<Self::Action>,
        state: &mut Self::State,
    ) -> Option<Self>;
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
    ) -> Option<Self> {
        match self {
            Action::Builtin(action) => {
                action.execute(bindings, context);
                None
            }
            Action::User(action) => action.execute(bindings, context, state).map(Action::User),
        }
    }
}

#[derive(Debug)]
pub(crate) struct PendingAction<A> {
    action: Action<A>,
    bindings: OwnedBindings,
}

impl<A> PendingAction<A> {
    pub(crate) fn new(action: Action<A>, bindings: OwnedBindings) -> Self {
        Self { action, bindings }
    }
}

impl<State, A> PendingAction<A>
where
    A: Execute<State = State, Action = A>,
{
    pub(crate) fn execute(self, context: &mut Context<A>, state: &mut State) -> Option<Self> {
        let Self { action, bindings } = self;
        let action = action.execute(&bindings, context, state)?;
        Some(Self { action, bindings })
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

#[cfg(test)]
mod tests {
    use alloc::vec;

    use ember_core::agent::Aid;
    use ember_core::message::Receiver;

    use crate::resolve::ResolveFailure;
    use crate::term::conversion::{ConversionError, FromTermError};
    use crate::term::view::TermView;
    use crate::testing::{bindings, string, variable};

    use super::*;

    #[test]
    fn test_variable_or_receiver_resolves_bound_variable_to_receiver() {
        let var = variable();
        let addr = string("receiver-agent@local");
        let bindings = bindings(vec![(var.clone(), TermView::from(&addr))]);

        let resolved = VariableOrReceiver::Variable(var)
            .resolve(&bindings)
            .expect("should resolve");

        assert_eq!(
            resolved,
            VariableOrReceiver::Receiver(Receiver::Single(Aid::local("receiver-agent")))
        );
    }

    #[test]
    fn test_variable_or_receiver_leaves_unbound_variable_unresolved() {
        let var = variable();
        let bindings = bindings(vec![]);

        let resolved = VariableOrReceiver::Variable(var.clone())
            .resolve(&bindings)
            .expect("should resolve");

        assert_eq!(resolved, VariableOrReceiver::Variable(var));
    }

    #[test]
    fn test_variable_or_receiver_fails_when_bound_value_is_not_an_aid() {
        let var = variable();
        let not_an_aid = string("not-an-aid");
        let bindings = bindings(vec![(var.clone(), TermView::from(&not_an_aid))]);

        let err = VariableOrReceiver::Variable(var)
            .resolve(&bindings)
            .unwrap_err();

        assert!(matches!(
            err,
            ResolveFailure::ConversionFailed(FromTermError::IncorrectConversion(
                ConversionError::InvalidAid(_)
            ))
        ));
    }

    #[test]
    fn test_variable_or_receiver_resolves_receiver_unchanged() {
        let receiver = Receiver::Single(Aid::local("receiver-agent"));
        let bindings = bindings(vec![]);

        let resolved = VariableOrReceiver::Receiver(receiver.clone())
            .resolve(&bindings)
            .expect("should resolve");

        assert_eq!(resolved, VariableOrReceiver::Receiver(receiver));
    }
}
