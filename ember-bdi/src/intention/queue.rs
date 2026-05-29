use alloc::collections::{BTreeMap, VecDeque};

use derive_where::derive_where;

use crate::bindings::Bindings;
use crate::context::Context;
use crate::plan::Plan;

use super::{Intention, IntentionId, IntentionRunResult};

#[derive(Debug)]
#[derive_where(Default)]
pub(crate) struct IntentionQueue<A> {
    intentions: BTreeMap<IntentionId, Intention<A>>,
    queue: VecDeque<IntentionId>,
    current_id: IntentionId,
}

impl<A> IntentionQueue<A> {
    fn next_id(&mut self) -> IntentionId {
        let id = self.current_id;
        self.current_id += 1;
        id
    }

    pub(crate) fn is_empty(&mut self) -> bool {
        self.queue.is_empty()
    }
}

impl<A: Clone> IntentionQueue<A> {
    pub(crate) fn push(
        &mut self,
        plan: &'_ Plan<A>,
        bindings: Bindings<'_>,
        existing_intention: Option<IntentionId>,
    ) {
        let id = existing_intention.unwrap_or_else(|| self.next_id());
        self.intentions.entry(id).or_default().push(plan, bindings);
        if !self.queue.contains(&id) {
            self.queue.push_back(id);
        }
    }

    pub(crate) fn step<S: Scheduler<A>>(&mut self, scheduler: &mut S, context: &mut Context<A>) {
        if let Some(intention) = self.next_intention(scheduler) {
            if let IntentionRunResult::Done = intention.step(context) {
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

                self.intentions
                    .remove(&id)
                    .expect("intention id should exist");
            }
        }
    }
}

impl<A> IntentionQueue<A> {
    fn next_intention<S: Scheduler<A>>(&mut self, scheduler: &mut S) -> Option<&mut Intention<A>> {
        let id = scheduler.select_intention(&self.queue, &self.intentions)?;
        Some(
            self.intentions
                .get_mut(&id)
                .expect("intention id should exist"),
        )
    }
}

pub trait Scheduler<A> {
    fn select_intention(
        &mut self,
        queue: &VecDeque<IntentionId>,
        intentions: &BTreeMap<IntentionId, Intention<A>>,
    ) -> Option<IntentionId>;
}

pub struct Fifo;

impl<A> Scheduler<A> for Fifo {
    fn select_intention<'b>(
        &mut self,
        queue: &VecDeque<IntentionId>,
        intentions: &BTreeMap<IntentionId, Intention<A>>,
    ) -> Option<IntentionId> {
        queue.iter().find(|i| intentions.contains_key(i)).copied()
    }
}
