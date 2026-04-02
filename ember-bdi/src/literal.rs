use crate::term::{Ground, NonGround, Structure};

// TODO: Support rules.
pub enum Literal<Groundness = NonGround> {
    Atom { negated: bool, structure: Structure },
    Variable(Groundness),
}

pub type GroundLiteral = Literal<Ground>;
