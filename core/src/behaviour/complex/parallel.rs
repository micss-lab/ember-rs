use alloc::collections::vec_deque::VecDeque;

use super::{Behaviour, BehaviourQueue};

pub struct ParallelBehaviour {
    queue: VecDeque<Behaviour>,
    finished: usize,
    strategy: Strategy,
}

pub enum Strategy {
    All,
    One,
    N(usize),
    Never,
}

impl ParallelBehaviour {
    pub fn new(strategy: Strategy) -> Self {
        Self {
            queue: VecDeque::new(),
            finished: 0,
            strategy,
        }
    }
}

impl BehaviourQueue for ParallelBehaviour {
    fn next(&mut self) -> Option<Behaviour> {
        self.queue.pop_front()
    }

    fn schedule(&mut self, behaviour: Behaviour) {
        self.queue.push_back(behaviour);
    }

    fn is_finished(&self) -> bool {
        match self.strategy {
            Strategy::All => self.queue.is_empty(),
            Strategy::One => self.finished >= 1,
            Strategy::N(n) => self.finished >= n,
            Strategy::Never => false,
        }
    }
}
