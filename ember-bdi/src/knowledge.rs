use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::literal::GroundLiteral;
use crate::term::Atom;

pub type Belief = GroundLiteral;

pub struct BeliefBase {
    /// Mapping from the belief atom and the arity to a list of ground truths.
    beliefs: BTreeMap<(Atom, usize), Vec<Belief>>,
}
