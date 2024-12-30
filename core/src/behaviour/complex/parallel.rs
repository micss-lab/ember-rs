use alloc::{boxed::Box, collections::vec_deque::VecDeque};

use super::macros::{complex_action_impl, complex_behaviour_methods};
use super::{
    Behaviour, BehaviourQueue, ComplexBehaviour, Context, IntoBehaviour, ScheduleStrategy,
};

pub trait ParallelBehaviour {
    type Message;

    type ChildMessage;

    fn initial_behaviours(&self) -> ParallelBehaviourQueue<Self::ChildMessage>;

    complex_behaviour_methods!();
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

impl<M: 'static> ParallelBehaviourQueue<M> {
    pub fn add_behaviour<K>(&mut self, behaviour: impl IntoBehaviour<K, Message = M>) {
        self.schedule(behaviour.into_behaviour(), ScheduleStrategy::End)
    }

    pub fn with_behaviour<K>(mut self, behaviour: impl IntoBehaviour<K, Message = M>) -> Self {
        self.add_behaviour(behaviour);
        self
    }
}

impl<M: 'static> BehaviourQueue<M> for ParallelBehaviourQueue<M> {
    fn next(&mut self) -> Option<Box<dyn Behaviour<Message = M>>> {
        self.queue.pop_front()
    }

    fn schedule(&mut self, behaviour: Box<dyn Behaviour<Message = M>>, strategy: ScheduleStrategy) {
        match strategy {
            ScheduleStrategy::Next => self.queue.push_front(behaviour),
            ScheduleStrategy::End => self.queue.push_back(behaviour),
        }
    }

    fn reschedule(&mut self, behaviour: Box<dyn Behaviour<Message = M>>) {
        self.schedule(behaviour, ScheduleStrategy::End);
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

struct ParallelBehaviourImpl<P: ParallelBehaviour>(P);

impl<P, M: 'static, CM: 'static> Behaviour
    for ComplexBehaviour<ParallelBehaviourImpl<P>, ParallelBehaviourQueue<CM>>
where
    P: ParallelBehaviour<Message = M, ChildMessage = CM> + 'static,
{
    type Message = M;

    complex_action_impl!();
}

#[doc(hidden)]
pub struct Parallel;

impl<T, M: 'static> IntoBehaviour<Parallel> for T
where
    T: ParallelBehaviour<Message = M> + 'static,
{
    type Message = M;

    fn into_behaviour(self) -> Box<dyn Behaviour<Message = Self::Message>> {
        let queue = self.initial_behaviours();
        Box::new(ComplexBehaviour {
            kind: ParallelBehaviourImpl(self),
            queue,
        })
    }
}
