use alloc::{boxed::Box, collections::vec_deque::VecDeque};

use super::{super::Behaviour, BehaviourQueue, ComplexBehaviour};

pub trait ParallelBehaviour {
    type Message;

    type ChildMessage;

    fn queue(&mut self) -> &mut ParallelBehaviourQueue<Self::ChildMessage>;
}

pub struct ParallelBehaviourQueue<M> {
    queue: VecDeque<Box<dyn Behaviour<Message = M>>>,
    finished: usize,
    strategy: FinishStrategy,
}

pub enum FinishStrategy {
    All,
    One,
    N(usize),
    Never,
}

impl<M> ParallelBehaviourQueue<M> {
    pub fn new(strategy: FinishStrategy) -> Self {
        Self {
            queue: VecDeque::new(),
            finished: 0,
            strategy,
        }
    }
}

impl<M: 'static> BehaviourQueue<M> for ParallelBehaviourQueue<M> {
    fn next(&mut self) -> Option<Box<dyn Behaviour<Message = M>>> {
        self.queue.pop_front()
    }

    fn schedule(&mut self, behaviour: Box<dyn Behaviour<Message = M>>) {
        self.queue.push_back(behaviour);
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

pub struct Par;
impl<T, M: 'static> ComplexBehaviour<M, Par> for T
where
    T: ParallelBehaviour<ChildMessage = M>,
{
    fn add_behaviour(&mut self, behaviour: impl Behaviour<Message = M>) {
        self.queue().schedule(Box::new(behaviour));
    }
}
