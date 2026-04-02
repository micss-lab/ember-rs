use alloc::vec::Vec;

use crate::literal::GroundedLiteral;

pub struct BeliefBase {
    // TODO: Improve this container type.
    beliefs: Vec<GroundedLiteral>,
}
