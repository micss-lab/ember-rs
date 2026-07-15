use alloc::boxed::Box;
use alloc::collections::BTreeSet;
use alloc::string::{String, ToString};

use bstr::BString;
use ember_util::cmp::TotalCmpF32;

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
        if let Some(args) = self.arguments.as_ref() {
            args.iter()
                .for_each(|arg| arg.collect_variables(&mut *variables))
        }
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
