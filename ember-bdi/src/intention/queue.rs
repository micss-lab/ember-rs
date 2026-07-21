use alloc::collections::{BTreeMap, BTreeSet, VecDeque};

use derive_where::derive_where;

use crate::bindings::{Bindings, OwnedBindings};
use crate::context::Context;
use crate::plan::Plan;

use super::result::*;
use super::{Intention, IntentionId};

#[derive(Debug)]
#[derive_where(Default)]
pub(crate) struct IntentionQueue<A> {
    intentions: BTreeMap<IntentionId, Intention<A>>,
    queue: VecDeque<IntentionId>,
    /// Intentions with an action that hasn't completed yet. The scheduler skips these until
    /// they're unblocked, so an intention never advances to its next formula while one of its
    /// actions is still being polled.
    blocked: BTreeSet<IntentionId>,
    current_id: IntentionId,
}

impl<A> IntentionQueue<A> {
    fn next_id(&mut self) -> IntentionId {
        let id = self.current_id;
        self.current_id += 1;
        id
    }

    pub(crate) fn block(&mut self, id: IntentionId) {
        self.blocked.insert(id);
    }

    pub(crate) fn unblock(&mut self, id: IntentionId) {
        self.blocked.remove(&id);
    }
}

impl<A: Clone> IntentionQueue<A> {
    pub(crate) fn push(
        &mut self,
        plan: &'_ Plan<A>,
        bindings: Bindings<'_>,
        existing_intention: Option<IntentionId>,
        event: crate::plan::TriggeringEvent,
    ) {
        let id = existing_intention.unwrap_or_else(|| self.next_id());
        self.intentions
            .entry(id)
            .or_insert_with(|| Intention::new(id))
            .push(plan, bindings, event);

        if !self.queue.contains(&id) {
            self.queue.push_back(id);
        }
    }

    pub(crate) fn step<'a, S: Scheduler<A>>(
        &'a mut self,
        scheduler: &mut S,
        context: &mut Context<A>,
    ) -> ReadOnlyBindings<'a> {
        let candidates = self
            .queue
            .iter()
            .copied()
            .filter(|id| !self.blocked.contains(id));

        let Some(id) = scheduler.select_intention(candidates, &self.intentions) else {
            return ReadOnlyBindings::Owned(OwnedBindings::empty());
        };

        let is_done = {
            let intention = self
                .intentions
                .get_mut(&id)
                .expect("intention id should exist");

            match intention.step(context) {
                Ok(StepOk::Pending) => false,
                Ok(StepOk::Done) => true,
                Err(_) => unimplemented!("report intention execution error to user"),
            }
        };

        if is_done {
            let intention = self
                .intentions
                .get_mut(&id)
                .expect("intention id should exist");

            let id = intention.id;
            // Remove the intention from the queue and from the intention-base.
            let idx = self
                .queue
                .iter()
                .enumerate()
                .find_map(|(i, e)| (*e == id).then_some(i))
                .expect("intention id should exist");

            let removed_id = self.queue.swap_remove_front(idx);
            debug_assert!(removed_id == Some(id));

            let mut intention = self
                .intentions
                .remove(&id)
                .expect("intention id should exist");

            ReadOnlyBindings::Owned(intention.take_last_bindings())
        } else {
            // TODO: Polonius (the new borrow checker) will fix the NLL limitation that prevents returning the
            // reference directly from the match arm above. Remove this lookup then.
            let intention = self.intentions.get(&id).expect("intention id should exist");

            intention
                .get_last_bindings()
                .map(ReadOnlyBindings::Borrowed)
                .unwrap_or_else(|| ReadOnlyBindings::Owned(OwnedBindings::empty()))
        }
    }
}

pub trait Scheduler<A> {
    fn select_intention(
        &mut self,
        candidates: impl IntoIterator<Item = IntentionId>,
        intentions: &BTreeMap<IntentionId, Intention<A>>,
    ) -> Option<IntentionId>;
}

pub struct Fifo;

impl<A> Scheduler<A> for Fifo {
    fn select_intention<'b>(
        &mut self,
        candidates: impl IntoIterator<Item = IntentionId>,
        intentions: &BTreeMap<IntentionId, Intention<A>>,
    ) -> Option<IntentionId> {
        candidates.into_iter().find(|i| intentions.contains_key(i))
    }
}
