use alloc::collections::{BTreeMap, BTreeSet};

use crate::context::Context;
use crate::literal::IntoLiteral;
use crate::plan::{Trigger, TriggeringEvent};
use crate::term::Atom;

use super::belief::Knowledge;
use super::query::{IntoQuery, Query};

#[derive(Debug, Default)]
pub struct KnowledgeBase {
    /// Mapping from the belief atom and the arity to a list of ground truths.
    pub(super) collections: BTreeMap<(Atom, usize), KnowledgeCollection>,
}

impl KnowledgeBase {
    /// Adds the belief to the knowledge-base and emits an event that the belief has been added/updated.
    /// Returns `true` if the belief was already present.
    pub fn assert<A>(&mut self, belief: impl IntoLiteral, context: &mut Context<A>) -> bool {
        let belief = belief.into_literal();
        let added = self.assert_no_event(belief.clone());
        if added {
            // TODO: Should this be an external event (no intention id)? I think it does...
            context.emit_event(
                TriggeringEvent {
                    trigger: Trigger::Addition,
                    event: belief,
                    goal: None,
                },
                None,
            );
        }
        added
    }

    /// Adds the knowledge to the knowledge-base without emitting an event for it. Returns `true` if the knowledge was
    /// already present.
    ///
    /// This method assumes that the update happened due to an event itself. Firing one again
    /// would result in an infinite loop.
    pub fn assert_no_event(&mut self, belief: impl Into<Knowledge>) -> bool {
        let belief = belief.into();
        let beliefs = self.collections.entry(belief.atom_and_arity()).or_default();
        beliefs.store(belief)
    }

    /// Removes the belief from the knowledge-base and emits an event that the belief has been removed/updated.
    /// Returns `true` if the belief has been removed.
    pub fn remove<A>(&mut self, belief: impl IntoLiteral, context: &mut Context<A>) -> bool {
        let belief = belief.into_literal();
        let removed = self.remove_no_event(belief.clone());
        if removed {
            // TODO: Should this be an external event (no intention id)? I think it does...
            context.emit_event(
                TriggeringEvent {
                    trigger: Trigger::Deletion,
                    event: belief,
                    goal: None,
                },
                None,
            );
        }
        removed
    }

    /// Removes the knowledge from the knowledge-base without emitting an event for it. Returns `true` if the knowledge has
    /// been removed.
    ///
    /// This method assumes that the update happened due to an event itself. Firing one again
    /// would result in an infinite loop.
    pub fn remove_no_event(&mut self, belief: impl Into<Knowledge>) -> bool {
        let belief = belief.into();
        let Some(beliefs) = self.collections.get_mut(&belief.atom_and_arity()) else {
            return false;
        };
        beliefs.remove(&belief)
    }

    pub fn query<'a>(&'a self, query: impl IntoQuery<'a>) -> Query<'a> {
        query.into_query(self)
    }
}

impl<B> FromIterator<B> for KnowledgeBase
where
    B: Into<Knowledge>,
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
pub(super) struct KnowledgeCollection(pub(super) BTreeSet<Knowledge>);

impl KnowledgeCollection {
    /// Stores the belief in the collection returning `true` if the belief is new.
    fn store(&mut self, belief: Knowledge) -> bool {
        self.0.insert(belief)
    }

    /// Removes the belief from the collection if it was previously stored.
    fn remove(&mut self, belief: &Knowledge) -> bool {
        self.0.remove(belief)
    }
}
