use alloc::boxed::Box;

use super::macros::{complex_action_impl, complex_behaviour_methods};
use super::queue::{BehaviourQueue, BehaviourScheduler, ScheduleStrategy};
use super::{get_id, Behaviour, BehaviourId, ComplexBehaviour, Context, IntoBehaviour};

pub trait SequentialBehaviour {
    type Event;

    type ChildEvent;

    fn initial_behaviours(&self) -> SequentialBehaviourQueue<Self::ChildEvent>;

    complex_behaviour_methods!();
}

pub struct SequentialBehaviourQueue<E> {
    queue: BehaviourQueue<E>,
}

impl<E: 'static> Default for SequentialBehaviourQueue<E> {
    fn default() -> Self {
        Self {
            queue: BehaviourQueue::new(),
        }
    }
}

impl<E: 'static> SequentialBehaviourQueue<E> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<E: 'static> SequentialBehaviourQueue<E> {
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

impl<E: 'static> BehaviourScheduler<E> for SequentialBehaviourQueue<E> {
    fn next(&mut self) -> Option<Box<dyn Behaviour<Event = E>>> {
        self.queue.pop()
    }

    fn schedule(&mut self, behaviour: Box<dyn Behaviour<Event = E>>, strategy: ScheduleStrategy) {
        self.queue.push(behaviour, strategy);
    }

    fn reschedule(&mut self, behaviour: Box<dyn Behaviour<Event = E>>) {
        self.schedule(behaviour, ScheduleStrategy::Next);
    }

    fn remove(&mut self, id: BehaviourId) -> bool {
        self.queue.remove(id)
    }

    fn block(&mut self, id: BehaviourId) -> bool {
        self.queue.block(id)
    }

    fn is_finished(&self) -> bool {
        self.queue.is_empty()
    }
}

struct SequentialBehaviourImpl<S: SequentialBehaviour>(S);

impl<S, E: 'static, CE: 'static> Behaviour
    for ComplexBehaviour<SequentialBehaviourImpl<S>, SequentialBehaviourQueue<CE>>
where
    S: SequentialBehaviour<Event = E, ChildEvent = CE> + 'static,
{
    type Event = E;

    fn id(&self) -> BehaviourId {
        self.id
    }

    complex_action_impl!();
}

#[doc(hidden)]
pub struct Sequential;

impl<T, E: 'static> IntoBehaviour<Sequential> for T
where
    T: SequentialBehaviour<Event = E> + 'static,
{
    type Event = E;

    fn into_behaviour(self) -> Box<dyn Behaviour<Event = Self::Event>> {
        let queue = self.initial_behaviours();
        Box::new(ComplexBehaviour {
            id: get_id(),
            kind: SequentialBehaviourImpl(self),
            queue,
        })
    }
}
