use alloc::boxed::Box;

use super::macros::{complex_action_impl, complex_behaviour_methods};
use super::queue::{BehaviourQueue, BehaviourScheduler, ScheduleStrategy};
use super::{get_id, Behaviour, BehaviourId, ComplexBehaviour, Context, IntoBehaviour};

pub trait SequentialBehaviour {
    type Message;

    type ChildMessage;

    fn initial_behaviours(&self) -> SequentialBehaviourQueue<Self::ChildMessage>;

    complex_behaviour_methods!();
}

pub struct SequentialBehaviourQueue<M> {
    queue: BehaviourQueue<M>,
}

impl<M: 'static> Default for SequentialBehaviourQueue<M> {
    fn default() -> Self {
        Self {
            queue: BehaviourQueue::new(),
        }
    }
}

impl<M: 'static> SequentialBehaviourQueue<M> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<M: 'static> SequentialBehaviourQueue<M> {
    pub fn add_behaviour<K>(
        &mut self,
        behaviour: impl IntoBehaviour<K, Message = M>,
    ) -> BehaviourId {
        let behaviour = behaviour.into_behaviour();
        let id = behaviour.id();
        self.schedule(behaviour, ScheduleStrategy::End);
        id
    }

    pub fn with_behaviour<K>(mut self, behaviour: impl IntoBehaviour<K, Message = M>) -> Self {
        self.add_behaviour(behaviour);
        self
    }
}

impl<M: 'static> BehaviourScheduler<M> for SequentialBehaviourQueue<M> {
    fn next(&mut self) -> Option<Box<dyn Behaviour<Message = M>>> {
        self.queue.pop()
    }

    fn schedule(&mut self, behaviour: Box<dyn Behaviour<Message = M>>, strategy: ScheduleStrategy) {
        self.queue.push(behaviour, strategy);
    }

    fn reschedule(&mut self, behaviour: Box<dyn Behaviour<Message = M>>) {
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

impl<S, M: 'static, CM: 'static> Behaviour
    for ComplexBehaviour<SequentialBehaviourImpl<S>, SequentialBehaviourQueue<CM>>
where
    S: SequentialBehaviour<Message = M, ChildMessage = CM> + 'static,
{
    type Message = M;

    fn id(&self) -> BehaviourId {
        self.id
    }

    complex_action_impl!();
}

#[doc(hidden)]
pub struct Sequential;

impl<T, M: 'static> IntoBehaviour<Sequential> for T
where
    T: SequentialBehaviour<Message = M> + 'static,
{
    type Message = M;

    fn into_behaviour(self) -> Box<dyn Behaviour<Message = Self::Message>> {
        let queue = self.initial_behaviours();
        Box::new(ComplexBehaviour {
            id: get_id(),
            kind: SequentialBehaviourImpl(self),
            queue,
        })
    }
}
