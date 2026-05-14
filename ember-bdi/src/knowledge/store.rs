use alloc::collections::btree_map::BTreeMap;

use crate::term::Atom;

use super::belief::{Belief, BeliefMetadata, NormalizedBelief};
use super::query::{IntoQuery, Query};

#[derive(Default)]
pub struct BeliefBase {
    /// Mapping from the belief atom and the arity to a list of ground truths.
    pub(super) beliefs: BTreeMap<(Atom, usize), BeliefCollection>,
}

impl BeliefBase {
    /// Adds the belief to the belief-base. Returns `true` if the belief was already present.
    pub fn assert(&mut self, belief: impl Into<Belief>) -> bool {
        let belief = belief.into();
        let beliefs = self.beliefs.entry(belief.atom_and_arity()).or_default();
        beliefs.store(belief)
    }

    /// Removes the belief from the belief-base. Returns `true` if the belief has been removed.
    pub fn remove(&mut self, belief: impl Into<Belief>) -> bool {
        let belief = belief.into();
        let Some(beliefs) = self.beliefs.get_mut(&belief.atom_and_arity()) else {
            return false;
        };
        beliefs.remove(belief)
    }

    pub fn query<'a>(&'a self, query: impl IntoQuery<'a>) -> Query<'a> {
        query.into_query(self)
    }
}

/// A collection of normalized beliefs. Beliefs are always stored normalized with additional
/// metadata on how to construct their original version.
#[derive(Debug, Default)]
pub(super) struct BeliefCollection(pub(super) BTreeMap<NormalizedBelief, BeliefMetadata>);

impl BeliefCollection {
    /// Stores the belief in the collection returning `true` if the belief was new.
    fn store(&mut self, belief: Belief) -> bool {
        use alloc::collections::btree_map::Entry;

        let (belief, metadata) = belief.normalize();
        match self.0.entry(belief) {
            Entry::Vacant(entry) => {
                entry.insert(metadata);
                true
            }
            Entry::Occupied(mut entry) => entry.get_mut().update(metadata),
        }
    }

    /// Removes the belief from the collection if it was previously stored.
    fn remove(&mut self, belief: Belief) -> bool {
        use alloc::collections::btree_map::Entry;

        let (belief, metadata) = belief.normalize();
        match self.0.entry(belief) {
            Entry::Vacant(_) => false,
            Entry::Occupied(mut entry) => {
                let (updated, should_remove) = entry.get_mut().remove(metadata);
                if should_remove {
                    entry.remove();
                }
                updated || should_remove
            }
        }
    }
}
