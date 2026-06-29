use crate::literal::{IntoLiteral, Literal};
use crate::plan::QueryFormula;
use crate::term::{Atom, Structure};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Belief {
    /// The head of the belief.
    pub(super) head: Literal,
    /// The rule associated to this belief.
    pub(super) rule: Option<QueryFormula>,
}

impl<L> From<L> for Belief
where
    L: IntoLiteral,
{
    fn from(literal: L) -> Self {
        Self::from_literal(literal)
    }
}

impl<L> From<(L, QueryFormula)> for Belief
where
    L: IntoLiteral,
{
    fn from((literal, rule): (L, QueryFormula)) -> Self {
        Self::from_literal_with_rule(literal, rule)
    }
}

impl Belief {
    fn from_literal<L>(literal: L) -> Self
    where
        L: IntoLiteral,
    {
        Self {
            head: literal.into_literal(),
            rule: None,
        }
    }

    fn from_literal_with_rule<L>(literal: L, rule: QueryFormula) -> Self
    where
        L: IntoLiteral,
    {
        Self {
            head: literal.into_literal(),
            rule: Some(rule),
        }
    }

    /// Return a reference to the internal [`Literal::Atom`] data. It is guaranteed that this exists as
    /// this is an invariant of the type. The return type can unfortunately not be a ground
    /// structure as it is not stored as such.
    fn as_literal_atom(&self) -> (&bool, &Structure) {
        todo!("this is no longer true");
        match &self.head {
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
