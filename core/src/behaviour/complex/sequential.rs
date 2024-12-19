use alloc::{boxed::Box, collections::vec_deque::VecDeque};

use super::{Behaviour, BehaviourQueue, ComplexBehaviour, ComplexBehaviourKind, IntoBehaviour};

pub trait SequentialBehaviour {
    type Message;

    type ChildMessage;

    fn queue(&mut self) -> &mut SequentialBehaviourQueue<Self::ChildMessage>;
}

pub struct SequentialBehaviourQueue<M> {
    queue: VecDeque<Box<dyn Behaviour<Message = M>>>,
}

impl<M> Default for SequentialBehaviourQueue<M> {
    fn default() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }
}

impl<M> SequentialBehaviourQueue<M> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<M: 'static> BehaviourQueue<M> for SequentialBehaviourQueue<M> {
    fn next(&mut self) -> Option<Box<dyn Behaviour<Message = M>>> {
        self.queue.pop_front()
    }

    fn schedule(&mut self, behaviour: Box<dyn Behaviour<Message = M>>) {
        self.queue.push_back(behaviour)
    }

    fn is_finished(&self) -> bool {
        self.queue.is_empty()
    }
}

#[doc(hidden)]
pub struct Sequential;

impl<T, M: 'static> ComplexBehaviour<M, Sequential> for T
where
    T: SequentialBehaviour<ChildMessage = M>,
{
    fn add_behaviour<K>(&mut self, behaviour: impl IntoBehaviour<K, Message = M>) {
        self.queue().schedule(behaviour.into_behaviour())
    }
}

impl<T, M: 'static> IntoBehaviour<Sequential> for T
where
    T: SequentialBehaviour<Message = M> + 'static,
{
    type Message = M;

    fn into_behaviour(self) -> Box<dyn Behaviour<Message = Self::Message>> {
        Box::new(ComplexBehaviourKind::Sequential(Box::new(self)))
    }
}
