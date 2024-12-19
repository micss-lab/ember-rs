use alloc::{boxed::Box, collections::vec_deque::VecDeque};

use crate::behaviour::Behaviour;

use super::{BehaviourQueue, ComplexBehaviour};

pub trait SequentialBehaviour<M> {
    type Message;

    fn queue(&mut self) -> &mut SequentialBehaviourQueue<M>;
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

pub struct Seq;
impl<T, M: 'static> ComplexBehaviour<M, Seq> for T
where
    T: SequentialBehaviour<M>,
{
    fn add_behaviour(&mut self, behaviour: impl Behaviour<Message = M>) {
        self.queue().schedule(Box::new(behaviour))
    }
}
