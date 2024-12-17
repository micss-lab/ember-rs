use alloc::collections::vec_deque::VecDeque;

use super::{Behaviour, BehaviourQueue};

#[derive(Default)]
pub struct SequentialBehaviour {
    queue: VecDeque<Behaviour>,
}

impl SequentialBehaviour {
    pub fn new() -> Self {
        Self::default()
    }
}

impl BehaviourQueue for SequentialBehaviour {
    fn next(&mut self) -> Option<Behaviour> {
        self.queue.pop_front()
    }

    fn schedule(&mut self, behaviour: Behaviour) {
        self.queue.push_back(behaviour)
    }

    fn is_finished(&self) -> bool {
        self.queue.is_empty()
    }
}
