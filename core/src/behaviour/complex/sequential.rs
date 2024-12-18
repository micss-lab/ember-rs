use alloc::{boxed::Box, collections::vec_deque::VecDeque};

use crate::behaviour::Behaviour;

use super::BehaviourQueue;

pub trait SequentialBehaviour<M>: Behaviour {
    fn queue(&mut self) -> &mut SequentialBehaviourQueue<M>;
}

pub struct SequentialBehaviourQueue<M> {
    queue: VecDeque<Box<dyn Behaviour<Message = M>>>,
}

impl<M> SequentialBehaviourQueue<M> {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }
}

struct Seq;
impl<M: 'static> BehaviourQueue<M> for SequentialBehaviourQueue<M> {
    type Ord = Seq;

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
