use alloc::vec::Vec;

use crate::literal::GroundLiteral;

pub struct BeliefBase {
    // TODO: Improve this container type.
    beliefs: Vec<GroundLiteral>,
}

impl FromIterator<GroundLiteral> for BeliefBase {
    fn from_iter<T: IntoIterator<Item = GroundLiteral>>(iter: T) -> Self {
        Self {
            beliefs: iter.into_iter().collect(),
        }
    }
}
