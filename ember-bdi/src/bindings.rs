use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::term::{Atom, NonGround, Structure, Term};
use crate::variable::{Variable, VariableId};

#[derive(Debug)]
pub struct Bindings<'a> {
    pub(crate) bindings: BTreeMap<VariableId, Option<TermView<'a>>>,
    pub(crate) aliases: AliasMap,
}

impl<'a> Bindings<'a> {
    pub(crate) fn new(
        bindings: impl IntoIterator<Item = (VariableId, Option<TermView<'a>>)>,
        aliases: AliasMap,
    ) -> Self {
        Self {
            bindings: bindings.into_iter().collect(),
            aliases,
        }
    }
}

impl<'a> Bindings<'a> {
    pub fn get(&self, variable: &Variable) -> Option<&TermView<'a>> {
        self.bindings.get(&variable.id)?.as_ref()
    }
}

#[derive(Debug, Default)]
pub(crate) struct AliasMap(Vec<(VariableId, VariableId)>);

impl AliasMap {
    pub(crate) fn new(aliases: impl IntoIterator<Item = (VariableId, VariableId)>) -> Self {
        Self(aliases.into_iter().collect())
    }

    pub(crate) fn iter(&self) -> core::slice::Iter<'_, (VariableId, VariableId)> {
        self.0.iter()
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TermView<'a> {
    Term(&'a Term),
    Literal {
        negated: bool,
        structure: StructureView<'a>,
    },
}

impl Clone for TermView<'_> {
    fn clone(&self) -> Self {
        match self {
            TermView::Term(term) => TermView::Term(term),
            TermView::Literal { negated, structure } => TermView::Literal {
                negated: *negated,
                structure: structure.clone(),
            },
        }
    }
}

impl<'a> From<&'a Term> for TermView<'a> {
    fn from(value: &'a Term) -> Self {
        Self::Term(value)
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
