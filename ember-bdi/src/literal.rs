use crate::term::{Ground, NonGround, Structure};

pub use ember_bdi_macros::IntoLiteral;

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

    pub(crate) fn variables(&self) -> alloc::vec::Vec<crate::variable::VariableId> {
        let mut vars = alloc::vec::Vec::new();
        self.collect_variables(&mut vars);
        vars
    }

    fn collect_variables(&self, vars: &mut alloc::vec::Vec<crate::variable::VariableId>) {
        match self {
            Literal::Atom { structure, .. } => {
                if let Some(args) = &structure.arguments {
                    for arg in args.iter() {
                        arg.collect_variables(vars);
                    }
                }
            }
            Literal::Variable(NonGround(v)) => vars.push(v.id),
        }
    }
}

pub trait IntoLiteral: Sized {
    fn into_literal(self) -> Literal<Ground>;
}
