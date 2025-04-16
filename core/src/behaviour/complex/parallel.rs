use alloc::boxed::Box;

use super::macros::{complex_action_impl, complex_behaviour_methods};
use super::queue::{BehaviourQueue, BehaviourScheduler, ScheduleStrategy};
use super::{get_id, Behaviour, BehaviourId, ComplexBehaviour, Context, IntoBehaviour};

pub trait ParallelBehaviour {
    type Event;

    type ChildEvent;

    fn initial_behaviours(&self) -> ParallelBehaviourQueue<Self::ChildEvent>;

    complex_behaviour_methods!();
}

pub struct ParallelBehaviourQueue<E> {
    queue: BehaviourQueue<E>,
    finished: usize,
    strategy: FinishStrategy,
}

pub enum FinishStrategy {
    All,
    One,
    N(usize),
    Never,
}

impl<E: 'static> ParallelBehaviourQueue<E> {
    pub fn new(strategy: FinishStrategy) -> Self {
        Self {
            queue: BehaviourQueue::new(),
            finished: 0,
            strategy,
        }
    }
}

impl<E: 'static> ParallelBehaviourQueue<E> {
    pub fn add_behaviour<K>(&mut self, behaviour: impl IntoBehaviour<K, Event = E>) -> BehaviourId {
        let behaviour = behaviour.into_behaviour();
        let id = behaviour.id();
        self.schedule(behaviour, ScheduleStrategy::End);
        id
    }

    pub fn with_behaviour<K>(mut self, behaviour: impl IntoBehaviour<K, Event = E>) -> Self {
        self.add_behaviour(behaviour);
        self
    }
}

impl<E: 'static> BehaviourScheduler<E> for ParallelBehaviourQueue<E> {
    fn next(&mut self) -> Option<Box<dyn Behaviour<Event = E>>> {
        self.queue.pop()
    }

    fn schedule(&mut self, behaviour: Box<dyn Behaviour<Event = E>>, strategy: ScheduleStrategy) {
        self.queue.push(behaviour, strategy)
    }

    fn reschedule(&mut self, behaviour: Box<dyn Behaviour<Event = E>>) {
        self.schedule(behaviour, ScheduleStrategy::End);
    }

    fn remove(&mut self, id: BehaviourId) -> bool {
        self.queue.remove(id)
    }

    fn block(&mut self, id: BehaviourId) -> bool {
        self.queue.block(id)
    }

    fn is_finished(&self) -> bool {
        match self.strategy {
            FinishStrategy::All => self.queue.is_empty(),
            FinishStrategy::One => self.finished >= 1,
            FinishStrategy::N(n) => self.finished >= n,
            FinishStrategy::Never => false,
        }
    }
}

struct ParallelBehaviourImpl<P: ParallelBehaviour>(P);

impl<P, E: 'static, CE: 'static> Behaviour
    for ComplexBehaviour<ParallelBehaviourImpl<P>, ParallelBehaviourQueue<CE>>
where
    P: ParallelBehaviour<Event = E, ChildEvent = CE> + 'static,
{
    type Event = E;

    fn id(&self) -> BehaviourId {
        self.id
    }

    complex_action_impl!();
}

#[doc(hidden)]
pub struct Parallel;

impl<T, E: 'static> IntoBehaviour<Parallel> for T
where
    T: ParallelBehaviour<Event = E> + 'static,
{
    type Event = E;

    fn into_behaviour(self) -> Box<dyn Behaviour<Event = Self::Event>> {
        let queue = self.initial_behaviours();
        Box::new(ComplexBehaviour {
            id: get_id(),
            kind: ParallelBehaviourImpl(self),
            queue,
        })
    }
}
