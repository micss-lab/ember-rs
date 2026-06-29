use crate::literal::{IntoLiteral, Literal};
use crate::plan::QueryFormula;
use crate::term::Atom;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Knowledge {
    /// The belief resulting from the Knowledge.
    pub(super) belief: Literal,
    /// The rule associated to this belief.
    pub(super) rule: Option<QueryFormula>,
}

impl<L> From<L> for Knowledge
where
    L: IntoLiteral,
{
    fn from(belief: L) -> Self {
        Self::from_belief(belief)
    }
}

impl<L> From<(L, QueryFormula)> for Knowledge
where
    L: IntoLiteral,
{
    fn from((belief, rule): (L, QueryFormula)) -> Self {
        Self::from_literal_with_rule(belief, rule)
    }
}

impl Knowledge {
    fn from_belief<L>(belief: L) -> Self
    where
        L: IntoLiteral,
    {
        Self {
            belief: belief.into_literal(),
            rule: None,
        }
    }

    fn from_literal_with_rule<L>(belief: L, rule: QueryFormula) -> Self
    where
        L: IntoLiteral,
    {
        Self {
            belief: belief.into_literal(),
            rule: Some(rule),
        }
    }

    pub(super) fn atom_and_arity(&self) -> (Atom, usize) {
        self.belief.structure.atom_and_arity()
    }
}
