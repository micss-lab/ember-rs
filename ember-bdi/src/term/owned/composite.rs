use alloc::boxed::Box;
use alloc::rc::Rc;

use ember_core::agent::Aid;
use ember_core::message::Receiver;

use crate::bindings::Bindings;
use crate::literal::Literal;
use crate::resolve::{Resolve, ResolveFailure};
use crate::term::reference::TermRef;
use crate::term::view::TermView;
use crate::variable::Variable;

use super::{Structure, Term};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariableOrReceiver {
    Variable(Variable),
    Receiver(Receiver),
}

impl Resolve for VariableOrReceiver {
    // TODO: a real borrowed view instead of cloning through `Self`, see `VariableOrList`.
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
    // TODO: a real borrowed view instead of cloning through `Self`, see `VariableOrList` - a
    // literal can grow large too.
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
                            .map(|args| args.iter().map(|t| t.to_owned()).collect()),
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum VariableOrList {
    Variable(Variable),
    List(Box<[Term]>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariableOrListView<'a> {
    Variable(Variable),
    List(Rc<[TermView<'a>]>),
}

impl VariableOrListView<'_> {
    fn to_owned(&self) -> VariableOrList {
        match self {
            VariableOrListView::Variable(v) => VariableOrList::Variable(v.clone()),
            VariableOrListView::List(items) => {
                VariableOrList::List(items.iter().map(TermView::to_owned).collect())
            }
        }
    }
}

impl Resolve for VariableOrList {
    // Kept a real borrowed view, unlike `VariableOrLiteral`/`VariableOrReceiver` above - a
    // `.findall` result feeding `.min`/`.max` is exactly the kind of list that can get large.
    type View<'a> = VariableOrListView<'a>;

    fn resolve(self, bindings: &Bindings<'_>) -> Result<Self, ResolveFailure> {
        Ok(self.resolve_as_view(bindings)?.to_owned())
    }

    fn resolve_as_view<'a>(
        &'a self,
        bindings: &Bindings<'a>,
    ) -> Result<Self::View<'a>, ResolveFailure> {
        Ok(match self {
            VariableOrList::Variable(v) => match bindings.lookup_view(v) {
                Some(TermView::List(items)) => VariableOrListView::List(items),
                Some(_) => return Err(ResolveFailure::IncorrectKind),
                None => VariableOrListView::Variable(v.clone()),
            },
            VariableOrList::List(items) => VariableOrListView::List(
                items
                    .iter()
                    .map(|t| t.resolve_as_view(bindings))
                    .collect::<Result<_, _>>()?,
            ),
        })
    }
}
