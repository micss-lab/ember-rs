use ember_collections::SmallSet;

pub use ember_bdi_macros::IntoLiteral;

use crate::term::view::StructureView;
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

    pub(crate) fn variables(&self) -> SmallSet<VariableId> {
        let mut vars = SmallSet::default();
        self.collect_variables(&mut vars);
        vars
    }

    pub(crate) fn collect_variables(&self, vars: &mut SmallSet<VariableId>) {
        self.structure.collect_variables(vars)
    }

    pub(crate) fn atom_and_arity(&self) -> (Atom, usize) {
        self.structure.atom_and_arity()
    }
}

impl core::fmt::Display for Literal {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.negated {
            write!(f, "~")?;
        }
        write!(f, "{}", self.structure)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LiteralView<'a> {
    pub negated: bool,
    pub structure: StructureView<'a>,
}

impl<'a> From<&'a Literal> for LiteralView<'a> {
    fn from(literal: &'a Literal) -> Self {
        let Literal {
            negated,
            ref structure,
        } = *literal;
        LiteralView {
            negated,
            structure: structure.into(),
        }
    }
}

impl<'a> From<&'a Structure> for LiteralView<'a> {
    fn from(structure: &'a Structure) -> Self {
        LiteralView {
            negated: false,
            structure: structure.into(),
        }
    }
}

impl LiteralView<'_> {
    pub(crate) fn to_owned(&self) -> Literal {
        let Self {
            negated,
            ref structure,
        } = *self;
        Literal {
            negated,
            structure: structure.to_owned(),
        }
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

impl IntoLiteral for LiteralView<'_> {
    fn into_literal(self) -> Literal {
        self.to_owned()
    }
}
