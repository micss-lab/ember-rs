use ember_core::agent::Aid;
use ember_core::message::Receiver;

use crate::bindings::Bindings;
use crate::literal::Literal;
use crate::resolve::{Resolve, ResolveFailure};
use crate::term::reference::TermRef;
use crate::variable::Variable;

use super::Structure;

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

    fn resolve(self, bindings: &Bindings<'_>) -> Result<Self, ResolveFailure> {
        Ok(match &self {
            VariableOrReceiver::Variable(v) => match bindings.lookup_as_type::<Aid>(v) {
                Some(Ok(aid)) => VariableOrReceiver::Receiver(Receiver::Single(aid)),
                Some(Err(e)) => return Err(ResolveFailure::ConversionFailed(e)),
                None => self,
            },
            VariableOrReceiver::Receiver(_) => self,
        })
    }

    fn resolve_as_view<'a>(
        &'a self,
        bindings: &Bindings<'a>,
    ) -> Result<Self::View<'a>, ResolveFailure> {
        self.clone().resolve(bindings)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariableOrLiteral {
    Variable(Variable),
    Literal(Literal),
}

impl Resolve for VariableOrLiteral {
    type View<'a>
        = Self
    where
        Self: 'a;

    fn resolve(self, bindings: &Bindings<'_>) -> Result<Self, ResolveFailure> {
        Ok(match self {
            VariableOrLiteral::Variable(v) => match bindings.lookup(&v) {
                Some(TermRef::Literal {
                    negated,
                    functor,
                    arguments,
                }) => Self::Literal(Literal {
                    negated,
                    structure: Structure {
                        functor: functor.clone(),
                        arguments: arguments
                            .map(|args| args.into_iter().map(|t| t.to_owned()).collect()),
                    },
                }),
                Some(_) => return Err(ResolveFailure::IncorrectKind),
                None => VariableOrLiteral::Variable(v),
            },
            // Its own arguments can still be unresolved variables, e.g. `mark_processed(X)`.
            VariableOrLiteral::Literal(lit) => Self::Literal(lit.resolve(bindings)?),
        })
    }

    fn resolve_as_view<'a>(
        &'a self,
        bindings: &Bindings<'a>,
    ) -> Result<Self::View<'a>, ResolveFailure> {
        self.clone().resolve(bindings)
    }
}
