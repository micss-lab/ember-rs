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
    String(&'a BStr),
    Variable(Variable),
    List(Rc<[TermRef<'a>]>),
    Literal {
        negated: bool,
        functor: &'a Atom,
        arguments: Option<Rc<[TermRef<'a>]>>,
    },
}

impl TermRef<'_> {
    pub fn to_owned(&self) -> Term {
        match self {
            Self::Number(n) => Term::Number(*n),
            Self::String(s) => Term::String((*s).into()),
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
                    arguments: arguments
                        .as_ref()
                        .map(|args| args.iter().map(|t| t.to_owned()).collect()),
                },
            }),
        }
    }
}

impl<'a> From<&'a Term> for TermRef<'a> {
    fn from(term: &'a Term) -> Self {
        match term {
            Term::Number(n) => Self::Number(*n),
            Term::String(s) => Self::String(s.as_ref()),
            Term::Variable(v) => Self::Variable(v.clone()),
            Term::List(items) => Self::List(items.iter().map(Into::into).collect()),
            &Term::Literal(Literal {
                negated,
                structure:
                    Structure {
                        ref functor,
                        ref arguments,
                    },
            }) => Self::Literal {
                negated,
                functor,
                arguments: arguments
                    .as_ref()
                    .map(|args| args.into_iter().map(Into::into).collect()),
            },
        }
    }
}

impl<'a> From<&'a TermView<'a>> for TermRef<'a> {
    fn from(term: &'a TermView<'a>) -> Self {
        match *term {
            TermView::Term(term) => term.into(),
            TermView::Number(n) => Self::Number(n),
            TermView::String(ref s) => s.into(),
            TermView::Variable(ref v) => Self::Variable(v.clone()),
            TermView::List(ref l) => Self::List(l.iter().map(Into::into).collect()),
            TermView::Literal(ref l) => l.into(),
        }
    }
}

impl<'a> From<&'a ViewString<'a>> for TermRef<'a> {
    fn from(string: &'a ViewString<'a>) -> Self {
        match string {
            ViewString::Borrowed(s) => Self::String(s),
            ViewString::Owned(s) => Self::String(s.as_ref()),
        }
    }
}

impl<'a> From<&'a LiteralView<'a>> for TermRef<'a> {
    fn from(
        LiteralView {
            negated,
            structure: StructureView { functor, arguments },
        }: &'a LiteralView<'a>,
    ) -> Self {
        Self::Literal {
            negated: *negated,
            functor: functor.as_ref(),
            arguments: arguments
                .as_ref()
                .map(|args| args.iter().map(Into::into).collect()),
        }
    }
}
