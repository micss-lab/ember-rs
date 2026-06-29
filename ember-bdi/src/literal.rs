use alloc::collections::btree_set::BTreeSet;

pub use ember_bdi_macros::IntoLiteral;

use crate::term::Structure;
use crate::variable::{Variable, VariableId};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Literal {
    Atom { negated: bool, structure: Structure },
    Variable(Variable),
}

impl Literal {
    pub fn is_ground(&self) -> bool {
        match self {
            Literal::Atom { structure, .. } => structure.is_ground(),
            Literal::Variable(_) => false,
        }
    }

    pub(crate) fn variables(&self) -> BTreeSet<VariableId> {
        let mut vars = BTreeSet::default();
        self.collect_variables(&mut vars);
        vars
    }

    fn collect_variables(&self, vars: &mut BTreeSet<VariableId>) {
        match self {
            Literal::Atom { structure, .. } => {
                if let Some(args) = &structure.arguments {
                    for arg in args.iter() {
                        arg.collect_variables(vars);
                    }
                }
            }
            Literal::Variable(v) => {
                vars.insert(v.id);
            }
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
