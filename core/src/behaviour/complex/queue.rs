use alloc::boxed::Box;
use alloc::collections::{BTreeSet, VecDeque};

use super::{Behaviour, BehaviourId, Context};

pub(super) struct BehaviourQueue<E> {
    behaviours: VecDeque<Box<dyn Behaviour<Event = E>>>,
    ids: BTreeSet<BehaviourId>,
    blocked_ids: BTreeSet<BehaviourId>,
}

impl<E> Default for BehaviourQueue<E> {
    fn default() -> Self {
        Self {
            behaviours: VecDeque::default(),
            ids: BTreeSet::default(),
            blocked_ids: BTreeSet::default(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ScheduleStrategy {
    Next,
    End,
}

impl<E: 'static> BehaviourQueue<E> {
    pub(super) fn new() -> Self {
        Self::default()
    }

    pub(super) fn push(
        &mut self,
        behaviour: Box<dyn Behaviour<Event = E>>,
        strategy: ScheduleStrategy,
    ) {
        let present = self.ids.insert(behaviour.id());
        if !present {
            return;
        }
        match strategy {
            ScheduleStrategy::Next => self.behaviours.push_front(behaviour),
            ScheduleStrategy::End => self.behaviours.push_back(behaviour),
        }
    }

    pub(super) fn pop(&mut self) -> Option<Box<dyn Behaviour<Event = E>>> {
        let mut amount = self.behaviours.len();
        Some(loop {
            let behaviour = self.behaviours.pop_front()?;
            let id = behaviour.id();

            // Next behaviour if it has been blocked.
            if self.blocked_ids.contains(&id) {
                self.behaviours.push_back(behaviour);
                continue;
            }

            // Check if the behaviour has not been removed yet.
            if self.ids.remove(&id) {
                break behaviour;
            }

            amount -= 1;
            if amount == 0 {
                // No behaviour is found that can be scheduled.
                return None;
            }
        })
    }

    pub(super) fn is_empty(&self) -> bool {
        self.behaviours.is_empty()
    }

    pub(super) fn remove(&mut self, id: BehaviourId) -> bool {
        self.ids.remove(&id)
    }

    pub(super) fn block(&mut self, id: BehaviourId) -> bool {
        if !self.ids.contains(&id) {
            return false;
        }
        self.blocked_ids.insert(id)
    }
}

pub(crate) trait BehaviourScheduler<E: 'static> {
    fn next(&mut self) -> Option<Box<dyn Behaviour<Event = E>>>;

    fn schedule(&mut self, behaviour: Box<dyn Behaviour<Event = E>>, strategy: ScheduleStrategy);

    fn reschedule(&mut self, behaviour: Box<dyn Behaviour<Event = E>>);

    fn remove(&mut self, id: BehaviourId) -> bool;

    fn block(&mut self, id: BehaviourId) -> bool;

    fn is_finished(&self) -> bool;

    fn action(&mut self, ctx: &mut Context<E>) -> bool {
        let Some(mut behaviour) = self.next() else {
            return self.is_finished();
        };
        let id = behaviour.id();

        let finished = behaviour.action(&mut *ctx);

        // Schedule newly created behaviours.
        if let Some(new_behaviours) = ctx.local.new_behaviours.take() {
            new_behaviours
                .into_iter()
                .flat_map(|(strategy, behaviours)| {
                    behaviours.into_iter().zip(core::iter::repeat(strategy))
                })
                .for_each(|(behaviour, strategy)| self.schedule(behaviour, strategy));
        }

        // Remove requested behaviours.
        ctx.local.removed_behaviours.drain(0..).for_each(|id| {
            self.remove(id);
        });

        if !finished {
            self.reschedule(behaviour);
        }

        // Block the current behaviour if requested.
        if ctx.local.should_block {
            self.block(id);
        }

        self.is_finished()
    }
}
