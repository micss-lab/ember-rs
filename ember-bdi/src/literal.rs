use alloc::collections::btree_set::BTreeSet;

pub use ember_bdi_macros::IntoLiteral;

use crate::term::{Atom, Structure};
use crate::variable::VariableId;

mod message;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Literal {
    pub negated: bool,
    pub structure: Structure,
}

impl Literal {
    pub fn is_ground(&self) -> bool {
        self.structure.is_ground()
    }

    pub(crate) fn variables(&self) -> BTreeSet<VariableId> {
        let mut vars = BTreeSet::default();
        self.collect_variables(&mut vars);
        vars
    }

    pub(crate) fn collect_variables(&self, vars: &mut BTreeSet<VariableId>) {
        self.structure.collect_variables(vars)
    }

    pub(crate) fn atom_and_arity(&self) -> (Atom, usize) {
        self.structure.atom_and_arity()
    }
}

pub trait IntoLiteral: Sized {
    fn into_literal(self) -> Literal;
}

impl IntoLiteral for Literal {
    fn into_literal(self) -> Literal {
        self
    }
}
