use crate::term::{Ground, NonGround, Structure, Term};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Literal<Groundness = NonGround> {
    Atom {
        negated: bool,
        structure: Structure<Groundness>,
    },
    Variable(Groundness),
}

pub type GroundLiteral = Literal<Ground>;

impl<G> From<Literal<G>> for Term<G> {
    fn from(value: Literal<G>) -> Self {
        match value {
            Literal::Atom { negated, structure } => Term::Literal { negated, structure },
            Literal::Variable(v) => Term::Variable(v),
        }
    }
}
