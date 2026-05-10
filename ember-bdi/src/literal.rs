use crate::term::{Ground, NonGround, Structure};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Literal<Groundness = NonGround> {
    Atom {
        negated: bool,
        structure: Structure<Groundness>,
    },
    Variable(Groundness),
}

pub type GroundLiteral = Literal<Ground>;

impl Literal<Ground> {
    pub fn into_non_ground(self) -> Literal<NonGround> {
        match self {
            Literal::Atom { negated, structure } => Literal::Atom {
                negated,
                structure: structure.into_non_ground(),
            },
            Literal::Variable(Ground(i)) => match i {},
        }
    }
}

impl Literal<NonGround> {
    pub fn try_into_ground(self) -> Option<Literal<Ground>> {
        match self {
            Literal::Atom { negated, structure } => Some(Literal::Atom {
                negated,
                structure: structure.try_into_ground()?,
            }),
            Literal::Variable(NonGround(_)) => None,
        }
    }
}
