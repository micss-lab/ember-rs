use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::literal::Literal;
use crate::term::{Atom, NonGround, Structure, Term, TotalCmpF32};
use crate::variable::{Variable, VariableId};

#[derive(Debug, Clone, Default)]
pub struct Bindings<'a> {
    pub(crate) bindings: Option<BTreeMap<VariableId, Option<TermView<'a>>>>,
    pub(crate) aliases: AliasMap,
}

impl<'a> Bindings<'a> {
    pub(crate) fn new(
        bindings: impl IntoIterator<Item = (VariableId, Option<TermView<'a>>)>,
        aliases: AliasMap,
    ) -> Self {
        Self {
            bindings: Some(bindings.into_iter().collect()),
            aliases,
        }
    }

    pub(crate) fn empty() -> Self {
        Self {
            bindings: None,
            aliases: AliasMap::empty(),
        }
    }
}

impl<'a> Bindings<'a> {
    pub fn get(&self, variable: &Variable) -> Option<&TermView<'a>> {
        self.bindings.as_ref()?.get(&variable.id)?.as_ref()
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct AliasMap(Vec<(VariableId, VariableId)>);

impl AliasMap {
    pub(crate) fn new(aliases: impl IntoIterator<Item = (VariableId, VariableId)>) -> Self {
        Self(aliases.into_iter().collect())
    }

    pub(crate) fn empty() -> Self {
        Self(Vec::with_capacity(0))
    }

    pub(crate) fn iter(&self) -> core::slice::Iter<'_, (VariableId, VariableId)> {
        self.0.iter()
    }
}

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
            Literal::Atom { negated, structure } => TermView::Literal { negated: *negated, structure: structure.into() },
            Literal::Variable(_) => todo!(),
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
