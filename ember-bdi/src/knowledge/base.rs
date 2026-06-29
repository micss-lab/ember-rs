use alloc::collections::{BTreeMap, BTreeSet};

use crate::context::Context;
use crate::plan::{Trigger, TriggeringEvent};
use crate::term::Atom;

use super::belief::Belief;
use super::query::{IntoQuery, Query};

#[derive(Debug, Default)]
pub struct BeliefBase {
    /// Mapping from the belief atom and the arity to a list of ground truths.
    pub(super) beliefs: BTreeMap<(Atom, usize), BeliefCollection>,
}

impl BeliefBase {
    /// Adds the belief to the belief-base and emits an event that the belief has been added/updated.
    /// Returns `true` if the belief was already present.
    pub fn assert<A>(&mut self, belief: impl Into<Belief>, context: &mut Context<A>) -> bool {
        let belief = belief.into();
        let added = self.assert_no_event(belief.clone());
        if added {
            // TODO: Should this be an external event (no intention id)? I think it does...
            context.emit_event(
                TriggeringEvent {
                    trigger: Trigger::Addition,
                    event: belief.into(),
                    goal: None,
                },
                None,
            );
        }
        added
    }

    /// Adds the belief to the belief-base without emitting an event for it. Returns `true` if the belief was
    /// already present.
    ///
    /// This method assumes that the update happened due to an event itself. Firing one again
    /// would result in an infinite loop.
    pub fn assert_no_event(&mut self, belief: impl Into<Belief>) -> bool {
        let belief = belief.into();
        let beliefs = self.beliefs.entry(belief.atom_and_arity()).or_default();
        beliefs.store(belief)
    }

    /// Removes the belief from the belief-base and emits an event that the belief has been removed/updated.
    /// Returns `true` if the belief has been removed.
    pub fn remove<A>(&mut self, belief: impl Into<Belief>, context: &mut Context<A>) -> bool {
        let belief = belief.into();
        let removed = self.remove_no_event(belief.clone());
        if removed {
            // TODO: Should this be an external event (no intention id)? I think it does...
            context.emit_event(
                TriggeringEvent {
                    trigger: Trigger::Deletion,
                    event: belief.into(),
                    goal: None,
                },
                None,
            );
        }
        removed
    }

    /// removed the belief to the belief-base without emitting an event for it. Returns `true` if the belief has
    /// been removed.
    ///
    /// This method assumes that the update happened due to an event itself. Firing one again
    /// would result in an infinite loop.
    pub fn remove_no_event(&mut self, belief: impl Into<Belief>) -> bool {
        let belief = belief.into();
        let Some(beliefs) = self.beliefs.get_mut(&belief.atom_and_arity()) else {
            return false;
        };
        beliefs.remove(&belief)
    }

    pub fn query<'a>(&'a self, query: impl IntoQuery<'a>) -> Query<'a> {
        query.into_query(self)
    }
}

impl<B> FromIterator<B> for BeliefBase
where
    B: Into<Belief>,
{
    fn from_iter<T: IntoIterator<Item = B>>(iter: T) -> Self {
        let mut this = Self::default();
        iter.into_iter().for_each(|b| {
            this.assert_no_event(b);
        });
        this
    }
}

/// A collection of beliefs not guaranteed to be semantically consistent.
#[derive(Debug, Default)]
pub(super) struct BeliefCollection(pub(super) BTreeSet<Belief>);

impl BeliefCollection {
    /// Stores the belief in the collection returning `true` if the belief is new.
    fn store(&mut self, belief: Belief) -> bool {
        self.0.insert(belief)
    }

    /// Removes the belief from the collection if it was previously stored.
    fn remove(&mut self, belief: &Belief) -> bool {
        self.0.remove(belief)
    }
}
