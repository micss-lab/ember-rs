use alloc::{boxed::Box, collections::vec_deque::VecDeque};

use super::{super::Behaviour, BehaviourQueue, ComplexBehaviour};

pub trait ParallelBehaviour<M>: Behaviour {
    fn queue(&mut self) -> &mut ParallelBehaviourQueue<M>;
}

pub struct ParallelBehaviourQueue<M> {
    queue: VecDeque<Box<dyn Behaviour<Message = M>>>,
    finished: usize,
    strategy: Strategy,
}

pub enum Strategy {
    All,
    One,
    N(usize),
    Never,
}

impl<M> ParallelBehaviourQueue<M> {
    pub fn new(strategy: Strategy) -> Self {
        Self {
            queue: VecDeque::new(),
            finished: 0,
            strategy,
        }
    }
}

struct Par;
impl<M: 'static> BehaviourQueue<M> for ParallelBehaviourQueue<M> {
    type Ord = Par;

    fn next(&mut self) -> Option<Box<dyn Behaviour<Message = M>>> {
        self.queue.pop_front()
    }

    fn schedule(&mut self, behaviour: Box<dyn Behaviour<Message = M>>) {
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
