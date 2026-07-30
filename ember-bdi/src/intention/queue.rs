use alloc::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::bindings::{Bindings, OwnedBindings};
use crate::context::Context;
use crate::plan::Plan;

use super::result::*;
use super::{Intention, IntentionId};

#[derive(Debug)]
pub(crate) struct IntentionQueue<A, Sched = Fifo> {
    intentions: BTreeMap<IntentionId, Intention<A>>,
    queue: VecDeque<IntentionId>,
    /// Intentions with an action that hasn't completed yet. The scheduler skips these until
    /// they're unblocked, so an intention never advances to its next formula while one of its
    /// actions is still being polled.
    blocked: BTreeSet<IntentionId>,
    current_id: IntentionId,
    scheduler: Sched,
}

impl<A, Sched: Default> Default for IntentionQueue<A, Sched> {
    fn default() -> Self {
        Self {
            intentions: BTreeMap::default(),
            queue: VecDeque::default(),
            blocked: BTreeSet::default(),
            current_id: 0,
            scheduler: Sched::default(),
        }
    }
}

impl<A, Sched> IntentionQueue<A, Sched> {
    fn next_id(&mut self) -> IntentionId {
        let id = self.current_id;
        self.current_id += 1;
        id
    }

    #[cfg(test)]
    pub(crate) fn is_empty(&mut self) -> bool {
        self.queue.is_empty()
    }

    pub(crate) fn block(&mut self, id: IntentionId) {
        self.blocked.insert(id);
    }

    pub(crate) fn unblock(&mut self, id: IntentionId) {
        self.blocked.remove(&id);
    }

    /// Whether there is at least one intention that isn't currently blocked, i.e. whether
    /// [`step`](Self::step) would actually advance anything right now.
    pub(crate) fn has_runnable(&self) -> bool {
        self.queue.iter().any(|id| !self.blocked.contains(id))
    }

    /// Configures which intention is stepped next when several are runnable. Replaces the
    /// default (`Fifo`). Changes the scheduler's type, so it returns a differently-typed queue.
    pub(crate) fn with_scheduler<NewSched>(self, scheduler: NewSched) -> IntentionQueue<A, NewSched> {
        IntentionQueue {
            intentions: self.intentions,
            queue: self.queue,
            blocked: self.blocked,
            current_id: self.current_id,
            scheduler,
        }
    }
}

impl<A: Clone, Sched> IntentionQueue<A, Sched> {
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

    pub(crate) fn step<'a>(&'a mut self, context: &mut Context<A>) -> ReadOnlyBindings<'a>
    where
        Sched: Scheduler<A>,
    {
        let candidates = self
            .queue
            .iter()
            .copied()
            .filter(|id| !self.blocked.contains(id));

        let Some(id) = self.scheduler.select_intention(candidates, &self.intentions) else {
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Fifo;

impl<A> Scheduler<A> for Fifo {
    fn select_intention(
        &mut self,
        candidates: impl IntoIterator<Item = IntentionId>,
        intentions: &BTreeMap<IntentionId, Intention<A>>,
    ) -> Option<IntentionId> {
        candidates.into_iter().find(|i| intentions.contains_key(i))
    }
}
