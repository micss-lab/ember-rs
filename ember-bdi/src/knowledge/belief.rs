use crate::literal::Literal;
use crate::plan::QueryFormula;
use crate::term::{Atom, Ground, NonGround, Structure};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Belief {
    /// While a belief is always a ground literal, storing one is done as a non-ground literal.
    /// This is done to allow borrowing internal data without first converting to a non-ground
    /// literal for unification.
    pub(super) literal: Literal<NonGround>,
    /// The rule associated to this belief.
    pub(super) rule: Option<QueryFormula>,
}

impl From<Literal<Ground>> for Belief {
    fn from(literal: Literal<Ground>) -> Self {
        Self::from_ground_literal(literal)
    }
}

impl From<(Literal<Ground>, QueryFormula)> for Belief {
    fn from((literal, rule): (Literal<Ground>, QueryFormula)) -> Self {
        Self::from_ground_literal_with_rule(literal, rule)
    }
}

impl From<Belief> for Literal<NonGround> {
    fn from(belief: Belief) -> Self {
        belief.literal
    }
}

impl Belief {
    fn from_ground_literal(literal: Literal<Ground>) -> Self {
        Self {
            literal: literal.into_non_ground(),
            rule: None,
        }
    }

    fn from_ground_literal_with_rule(literal: Literal<Ground>, rule: QueryFormula) -> Self {
        Self {
            literal: literal.into_non_ground(),
            rule: Some(rule),
        }
    }

    /// Return a reference to the internal [`Literal::Atom`] data. It is guaranteed that this exists as
    /// this is an invariant of the type. The return type can unfortunately not be a ground
    /// structure as it is not stored as such.
    fn as_literal_atom(&self) -> (&bool, &Structure<NonGround>) {
        match &self.literal {
            Literal::Atom { negated, structure } => (negated, structure),
            Literal::Variable(_) => {
                unreachable!("belief is always a ground literal")
            }
        }
    }

    pub(super) fn atom_and_arity(&self) -> (Atom, usize) {
        let (_, structure) = self.as_literal_atom();
        structure.atom_and_arity()
    }
}
