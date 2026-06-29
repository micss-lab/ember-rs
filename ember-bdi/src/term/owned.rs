use alloc::boxed::Box;
use alloc::collections::BTreeSet;
use alloc::string::{String, ToString};

use bstr::BString;

use crate::literal::Literal;
use crate::variable::{Variable, VariableId};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Term {
    // TODO: Support full arithmetic formulas here.
    Number(TotalCmpF32),
    String(BString),
    Variable(Variable),
    // TODO: Support lists.
    // List(List),
    Literal(Literal),
}

impl Term {
    pub fn is_ground(&self) -> bool {
        use Term::*;
        match self {
            Number(_) | String(_) => true,
            Variable(_) => false,
            Literal(literal) => literal.is_ground(),
        }
    }

    pub(crate) fn collect_variables(&self, vars: &mut BTreeSet<VariableId>) {
        match self {
            Term::Variable(v) => {
                vars.insert(v.id);
            }
            Term::Literal(literal) => literal.collect_variables(vars),
            _ => {}
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TotalCmpF32(f32);

impl From<f32> for TotalCmpF32 {
    fn from(value: f32) -> Self {
        Self(value)
    }
}

impl core::ops::Deref for TotalCmpF32 {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for TotalCmpF32 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl PartialOrd for TotalCmpF32 {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TotalCmpF32 {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl PartialEq for TotalCmpF32 {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl Eq for TotalCmpF32 {}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Structure {
    pub functor: Atom,
    pub arguments: Option<Box<[Term]>>,
}

impl Structure {
    pub fn is_ground(&self) -> bool {
        let Structure { arguments, .. } = self;
        arguments
            .as_ref()
            .map(|args| args.iter().all(|a| a.is_ground()))
            .unwrap_or(true)
    }

    pub(crate) fn collect_variables(&self, variables: &mut BTreeSet<VariableId>) {
        self.arguments.as_ref().map(|args| {
            args.iter()
                .for_each(|arg| arg.collect_variables(&mut *variables))
        });
    }

    pub(crate) fn atom_and_arity(&self) -> (Atom, usize) {
        (
            self.functor.clone(),
            self.arguments.as_ref().map(|args| args.len()).unwrap_or(0),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Atom(pub String);

impl<T> From<T> for Atom
where
    T: ToString,
{
    fn from(value: T) -> Self {
        Self(value.to_string())
    }
}
