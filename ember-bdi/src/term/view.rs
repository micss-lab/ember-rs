use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::literal::Literal;
use crate::variable::Variable;

use super::{Atom, NonGround, Structure, Term, TotalCmpF32};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TermView<'a> {
    Term(&'a Term),
    Number(TotalCmpF32),
    Variable(&'a Variable),
    Literal {
        negated: bool,
        structure: StructureView<'a>,
    },
}

impl Clone for TermView<'_> {
    fn clone(&self) -> Self {
        match self {
            Self::Term(term) => Self::Term(term),
            Self::Number(n) => Self::Number(*n),
            Self::Variable(v) => Self::Variable(v),
            Self::Literal { negated, structure } => Self::Literal {
                negated: *negated,
                structure: structure.clone(),
            },
        }
    }
}

impl<'a> From<&'a Term> for TermView<'a> {
    fn from(value: &'a Term) -> Self {
        match value {
            Term::Number(n) => TermView::Number(*n),
            t => TermView::Term(t),
        }
    }
}

impl<'a> From<&'a Literal> for TermView<'a> {
    fn from(literal: &'a Literal) -> Self {
        match literal {
            Literal::Atom { negated, structure } => TermView::Literal {
                negated: *negated,
                structure: structure.into(),
            },
            Literal::Variable(NonGround(v)) => TermView::Variable(v),
        }
    }
}

impl<'a> TermView<'a> {
    pub(crate) fn as_variable(&self) -> Option<&'a Variable> {
        let Self::Term(term) = self else {
            return None;
        };
        let Term::Variable(NonGround(v)) = term else {
            return None;
        };
        Some(v)
    }
}

impl TermView<'_> {
    pub(crate) fn to_owned(&self) -> Term {
        match *self {
            TermView::Term(term) => term.clone(),
            TermView::Number(n) => Term::Number(n),
            TermView::Variable(v) => Term::Variable(NonGround(v.clone())),
            TermView::Literal {
                negated,
                ref structure,
            } => Term::Literal {
                negated,
                structure: structure.to_owned(),
            },
        }
    }
}

impl Term {
    pub(crate) fn as_view(&self) -> TermView {
        self.into()
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct StructureView<'a> {
    pub functor: &'a Atom,
    pub arguments: Option<Box<[TermView<'a>]>>,
}

impl Clone for StructureView<'_> {
    fn clone(&self) -> Self {
        Self {
            functor: self.functor,
            arguments: self.arguments.clone(),
        }
    }
}

impl<'a> From<&'a Structure> for StructureView<'a> {
    fn from(Structure { functor, arguments }: &'a Structure) -> Self {
        Self {
            functor,
            arguments: arguments.as_ref().map(|a| {
                a.iter()
                    .map(|t| t.into())
                    .collect::<Vec<_>>()
                    .into_boxed_slice()
            }),
        }
    }
}

impl<'a> From<&'a Structure> for TermView<'a> {
    fn from(structure: &'a Structure) -> Self {
        Self::Literal {
            negated: false,
            structure: structure.into(),
        }
    }
}

impl StructureView<'_> {
    pub(crate) fn to_owned(&self) -> Structure {
        Structure {
            functor: self.functor.clone(),
            arguments: self.arguments.as_ref().map(|ts| {
                ts.into_iter()
                    .map(|t| t.to_owned())
                    .collect::<Vec<_>>()
                    .into_boxed_slice()
            }),
        }
    }
}
