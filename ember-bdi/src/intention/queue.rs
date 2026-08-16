use alloc::collections::BTreeMap;

use crate::bindings::Bindings;
use crate::context::Context;
use crate::knowledge::base::KnowledgeBase;
use crate::plan::action::Execute;
use crate::plan::{Plan, TriggeringEvent};

use super::result::*;
use super::{Intention, IntentionId};

#[derive(Debug)]
pub(crate) struct IntentionQueue<A, Sched = Random> {
    intentions: BTreeMap<IntentionId, Intention<A>>,
    blocked: BTreeMap<IntentionId, BlockReasons>,
    current_id: IntentionId,
    scheduler: Sched,
}

impl<A, Sched: Default> Default for IntentionQueue<A, Sched> {
    fn default() -> Self {
        Self {
            intentions: BTreeMap::default(),
            blocked: BTreeMap::default(),
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
        self.intentions.is_empty()
    }

    #[cfg(test)]
    pub(crate) fn intention_stack_len(&self, id: IntentionId) -> Option<usize> {
        self.intentions.get(&id).map(Intention::stack_len)
    }

    /// Blocks `id` on an event it just raised. Must be paired with a later
    /// [`unblock_event`](Self::unblock_event) once that event has been processed.
    pub(crate) fn block_on_event(&mut self, id: IntentionId) {
        self.blocked.entry(id).or_default().pending_events += 1;
    }

    /// Lifts one event block previously placed by [`block_on_event`](Self::block_on_event).
    pub(crate) fn unblock_event(&mut self, id: IntentionId) {
        let reasons = self
            .blocked
            .get_mut(&id)
            .expect("intention wasn't blocked on an event");
        reasons.pending_events = reasons
            .pending_events
            .checked_sub(1)
            .expect("intention wasn't blocked on an event");
        if !reasons.is_blocked() {
            self.blocked.remove(&id);
        }
    }

    /// Blocks `id` on an action it just dispatched. Must be paired with a later
    /// [`unblock_action`](Self::unblock_action) once that action finishes.
    pub(crate) fn block_on_action(&mut self, id: IntentionId) {
        let reasons = self.blocked.entry(id).or_default();
        assert!(
            !reasons.pending_action,
            "intention already has an action blocking it"
        );
        reasons.pending_action = true;
    }

    /// Lifts the action block previously placed by [`block_on_action`](Self::block_on_action).
    pub(crate) fn unblock_action(&mut self, id: IntentionId) {
        let reasons = self
            .blocked
            .get_mut(&id)
            .expect("intention wasn't blocked on an action");
        assert!(
            reasons.pending_action,
            "intention wasn't blocked on an action"
        );
        reasons.pending_action = false;
        if !reasons.is_blocked() {
            self.blocked.remove(&id);
        }
    }

    /// Whether there is at least one intention that isn't currently blocked, i.e. whether
    /// [`step`](Self::step) would actually advance anything right now.
    pub(crate) fn has_runnable(&self) -> bool {
        self.intentions
            .keys()
            .any(|id| !self.blocked.contains_key(id))
    }

    /// Configures which intention is stepped next when several are runnable. Replaces the
    /// default (`Random`). Changes the scheduler's type, so it returns a differently-typed queue.
    pub(crate) fn with_scheduler<NewSched>(
        self,
        scheduler: NewSched,
    ) -> IntentionQueue<A, NewSched> {
        IntentionQueue {
            intentions: self.intentions,
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
        event: TriggeringEvent,
    ) {
        let id = existing_intention.unwrap_or_else(|| self.next_id());
        self.intentions
            .entry(id)
            .or_insert_with(|| Intention::new(id))
            .push(plan, bindings, event);
    }
}

impl<A, S, Sched> IntentionQueue<A, Sched>
where
    A: Execute<State = S, UserAction = A>,
{
    pub(crate) fn step(
        &mut self,
        context: &mut Context<A>,
        knowledge: &mut KnowledgeBase,
        state: &mut S,
    ) where
        Sched: Scheduler<A>,
    {
        let candidates = self
            .intentions
            .keys()
            .copied()
            .filter(|id| !self.blocked.contains_key(id));

        let Some(id) = self
            .scheduler
            .select_intention(candidates, &self.intentions)
        else {
            return;
        };

        let is_done = {
            let intention = self
                .intentions
                .get_mut(&id)
                .expect("intention id should exist");

            match intention.step(context, knowledge, state) {
                Ok(StepOk::Pending) => false,
                Ok(StepOk::Done) => true,
                Err(_) => unimplemented!("report intention execution error to user"),
            }
        };

        if is_done {
            self.intentions
                .remove(&id)
                .expect("intention id should exist");
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct BlockReasons {
    pending_events: usize,
    /// Whether it has dispatched an action that hasn't finished executing yet.
    pending_action: bool,
}

impl BlockReasons {
    fn is_blocked(&self) -> bool {
        self.pending_events > 0 || self.pending_action
    }
}

pub trait Scheduler<A> {
    fn select_intention(
        &mut self,
        candidates: impl IntoIterator<Item = IntentionId>,
        intentions: &BTreeMap<IntentionId, Intention<A>>,
    ) -> Option<IntentionId>;
}

/// Favours the longest running intention until it has run to completion.
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

/// Picks a runnable intention (roughly) uniformly at random each step instead of always
/// favouring the oldest one, so a long-running intention doesn't starve the others under a
/// small `max_intentions` budget. Uses a fixed-seed xorshift64 PRNG.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Random {
    state: u64,
}

impl Default for Random {
    fn default() -> Self {
        // Xorshift64 needs a non-zero seed. The exact value doesn't matter.
        Self {
            state: 0x9E3779B97F4A7C15,
        }
    }
}

impl Random {
    /// Advances the PRNG and returns the next pseudo-random value.
    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }
}

impl<A> Scheduler<A> for Random {
    fn select_intention(
        &mut self,
        candidates: impl IntoIterator<Item = IntentionId>,
        intentions: &BTreeMap<IntentionId, Intention<A>>,
    ) -> Option<IntentionId> {
        // Reservoir sampling (k = 1): picks one candidate uniformly at random in a single pass,
        // without needing to know the candidate count up front.
        let mut chosen = None;
        let mut count: u64 = 0;

        for id in candidates
            .into_iter()
            .filter(|id| intentions.contains_key(id))
        {
            count += 1;
            if self.next_u64().is_multiple_of(count) {
                chosen = Some(id);
            }
        }

        chosen
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use crate::bindings::Bindings;
    use crate::testing::{plan, trigger};

    use super::*;

    /// A queue with a single, freshly pushed, unblocked intention.
    fn queue_with_one_intention() -> IntentionQueue<()> {
        let mut queue = IntentionQueue::default();
        let body = plan::<()>(trigger("start", vec![], None), None, vec![]);
        queue.push(
            &body,
            Bindings::empty(),
            None,
            trigger("start", vec![], None),
        );
        queue
    }

    #[test]
    fn intention_stays_blocked_until_every_reason_clears() {
        let mut queue = queue_with_one_intention();
        let id = *queue.intentions.keys().next().unwrap();

        // A belief update it triggered on the way to dispatching an action can raise an event
        // of its own - the two reasons are independent and must both clear.
        queue.block_on_action(id);
        queue.block_on_event(id);
        assert!(!queue.has_runnable());

        queue.unblock_event(id);
        assert!(
            !queue.has_runnable(),
            "the action block is still outstanding"
        );

        queue.unblock_action(id);
        assert!(queue.has_runnable(), "both reasons cleared");
    }

    #[test]
    #[should_panic]
    fn unblocking_a_reason_that_was_never_set_panics() {
        let mut queue = queue_with_one_intention();
        let id = *queue.intentions.keys().next().unwrap();

        queue.unblock_action(id);
    }
}
