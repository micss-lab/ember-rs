use alloc::boxed::Box;
use alloc::collections::{BTreeSet, VecDeque};

use super::{Behaviour, BehaviourId, Context};

pub(super) struct BehaviourQueue<M> {
    behaviours: VecDeque<Box<dyn Behaviour<Message = M>>>,
    ids: BTreeSet<BehaviourId>,
}

impl<M> Default for BehaviourQueue<M> {
    fn default() -> Self {
        Self {
            behaviours: VecDeque::default(),
            ids: BTreeSet::default(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ScheduleStrategy {
    Next,
    End,
}

impl<M: 'static> BehaviourQueue<M> {
    pub(super) fn new() -> Self {
        Self::default()
    }

    pub(super) fn push(
        &mut self,
        behaviour: Box<dyn Behaviour<Message = M>>,
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

    pub(super) fn pop(&mut self) -> Option<Box<dyn Behaviour<Message = M>>> {
        Some(loop {
            let behaviour = self.behaviours.pop_front()?;
            // Check if the behaviour has not been removed yet.
            if self.ids.remove(&behaviour.id()) {
                break behaviour;
            }
        })
    }

    pub(super) fn is_empty(&self) -> bool {
        self.behaviours.is_empty()
    }

    pub(super) fn remove(&mut self, id: BehaviourId) -> bool {
        self.ids.remove(&id)
    }
}

pub(crate) trait BehaviourScheduler<M: 'static> {
    fn next(&mut self) -> Option<Box<dyn Behaviour<Message = M>>>;

    fn schedule(&mut self, behaviour: Box<dyn Behaviour<Message = M>>, strategy: ScheduleStrategy);

    fn reschedule(&mut self, behaviour: Box<dyn Behaviour<Message = M>>);

    fn remove(&mut self, id: BehaviourId) -> bool;

    fn is_finished(&self) -> bool;

    fn action(&mut self, ctx: &mut Context<M>) -> bool {
        let Some(mut behaviour) = self.next() else {
            return self.is_finished();
        };
        let finished = behaviour.action(&mut *ctx);

        // Immediatly schedule newly created behaviours.
        if let Some(new_behaviours) = ctx.new_behaviours.take() {
            new_behaviours
                .into_iter()
                .flat_map(|(strategy, behaviours)| {
                    behaviours.into_iter().zip(core::iter::repeat(strategy))
                })
                .for_each(|(behaviour, strategy)| self.schedule(behaviour, strategy));
        }

        if !finished {
            self.reschedule(behaviour);
        }
        self.is_finished()
    }
}
