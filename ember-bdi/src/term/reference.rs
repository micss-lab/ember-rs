use alloc::borrow::{Cow, ToOwned};
use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::vec::Vec;

use bstr::BStr;
use ember_util::cmp::TotalCmpF32;

use crate::literal::{Literal, LiteralView};
use crate::variable::Variable;

use super::owned::{Atom, Structure, Term};
use super::view::{StructureView, TermView, ViewString};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TermRef<'a> {
    Number(TotalCmpF32),
    String(Cow<'a, BStr>),
    Variable(Variable),
    List(Box<[TermRef<'a>]>),
    Literal {
        negated: bool,
        functor: Rc<Atom>,
        arguments: Box<[TermRef<'a>]>,
    },
}

impl TermRef<'_> {
    pub fn to_owned(&self) -> Term {
        match self {
            Self::Number(n) => Term::Number(*n),
            Self::String(s) => Term::String(s.clone().into_owned()),
            Self::Variable(v) => Term::Variable(v.clone()),
            Self::List(items) => Term::List(
                items
                    .iter()
                    .map(TermRef::to_owned)
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            ),
            Self::Literal {
                negated,
                functor,
                arguments,
            } => Term::Literal(Literal {
                negated: *negated,
                structure: Structure {
                    functor: (**functor).clone(),
                    arguments: (!arguments.is_empty()).then(|| {
                        arguments
                            .iter()
                            .map(|t| t.to_owned())
                            .collect::<Vec<_>>()
                            .into_boxed_slice()
                    }),
                },
            }),
        }
    }
}

impl<'a> From<&'a Term> for TermRef<'a> {
    fn from(term: &'a Term) -> Self {
        match term {
            Term::Number(n) => Self::Number(*n),
            Term::String(s) => Self::String(Cow::Borrowed(s.as_ref())),
            Term::Variable(v) => Self::Variable(v.clone()),
            Term::List(items) => Self::List(
                items
                    .iter()
                    .map(Into::into)
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            ),
            &Term::Literal(Literal {
                negated,
                structure:
                    Structure {
                        ref functor,
                        ref arguments,
                    },
            }) => Self::Literal {
                negated,
                functor: Rc::new(functor.clone()),
                arguments: arguments
                    .as_ref()
                    .map(|args| {
                        args.into_iter()
                            .map(Into::into)
                            .collect::<Vec<_>>()
                            .into_boxed_slice()
                    })
                    .unwrap_or_default(),
            },
        }
    }
}

impl<'a> From<TermView<'a>> for TermRef<'a> {
    fn from(term: TermView<'a>) -> Self {
        match term {
            TermView::Term(term) => Self::from(term),
            TermView::Number(n) => Self::Number(n),
            TermView::String(ViewString::Borrowed(s)) => Self::String(Cow::Borrowed(s)),
            TermView::String(ViewString::Owned(s)) => Self::String(Cow::Owned((*s).to_owned())),
            TermView::Variable(v) => Self::Variable(v),
            TermView::List(items) => Self::List(
                items
                    .iter()
                    .cloned()
                    .map(Into::into)
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            ),
            TermView::Literal(literal) => literal.into(),
        }
    }
}

impl<'a> From<LiteralView<'a>> for TermRef<'a> {
    fn from(
        LiteralView {
            negated,
            structure: StructureView { functor, arguments },
        }: LiteralView<'a>,
    ) -> Self {
        Self::Literal {
            negated,
            functor,
            arguments: arguments
                .map(|args| {
                    args.iter()
                        .cloned()
                        .map(Into::into)
                        .collect::<Vec<_>>()
                        .into_boxed_slice()
                })
                .unwrap_or_default(),
        }
    }
}
