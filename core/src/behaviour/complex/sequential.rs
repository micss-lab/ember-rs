use alloc::boxed::Box;
use alloc::collections::vec_deque::VecDeque;

use super::{
    Behaviour, BehaviourQueue, ComplexBehaviour, ComplexBehaviourKind, Context, IntoBehaviour,
};

pub trait SequentialBehaviour {
    type Message;

    type ChildMessage;

    fn initial_behaviours(&self) -> SequentialBehaviourQueue<Self::ChildMessage>;

    fn after_child_action(&mut self, ctx: &mut Context<Self::Message>) {
        let _ = ctx;
    }
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

impl<M: 'static> SequentialBehaviourQueue<M> {
    pub fn add_behaviour<K>(&mut self, behaviour: impl IntoBehaviour<K, Message = M>) {
        self.schedule(behaviour.into_behaviour())
    }

    pub fn with_behaviour<K>(mut self, behaviour: impl IntoBehaviour<K, Message = M>) -> Self {
        self.add_behaviour(behaviour);
        self
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

impl<T, M: 'static> IntoBehaviour<Sequential> for T
where
    T: SequentialBehaviour<Message = M> + 'static,
{
    type Message = M;

    fn into_behaviour(self) -> Box<dyn Behaviour<Message = Self::Message>> {
        let queue = self.initial_behaviours();
        Box::new(ComplexBehaviour {
            kind: ComplexBehaviourKind::Sequential(Box::new(self)),
            queue,
        })
    }
}
