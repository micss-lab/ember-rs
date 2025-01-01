use alloc::boxed::Box;

use super::macros::{complex_action_impl, complex_behaviour_methods};
use super::queue::{BehaviourQueue, BehaviourScheduler, ScheduleStrategy};
use super::{get_id, Behaviour, BehaviourId, ComplexBehaviour, Context, IntoBehaviour};

pub trait ParallelBehaviour {
    type Message;

    type ChildMessage;

    fn initial_behaviours(&self) -> ParallelBehaviourQueue<Self::ChildMessage>;

    complex_behaviour_methods!();
}

pub struct ParallelBehaviourQueue<M> {
    queue: BehaviourQueue<M>,
    finished: usize,
    strategy: FinishStrategy,
}

pub enum FinishStrategy {
    All,
    One,
    N(usize),
    Never,
}

impl<M: 'static> ParallelBehaviourQueue<M> {
    pub fn new(strategy: FinishStrategy) -> Self {
        Self {
            queue: BehaviourQueue::new(),
            finished: 0,
            strategy,
        }
    }
}

impl<M: 'static> ParallelBehaviourQueue<M> {
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

impl<M: 'static> BehaviourScheduler<M> for ParallelBehaviourQueue<M> {
    fn next(&mut self) -> Option<Box<dyn Behaviour<Message = M>>> {
        self.queue.pop()
    }

    fn schedule(&mut self, behaviour: Box<dyn Behaviour<Message = M>>, strategy: ScheduleStrategy) {
        self.queue.push(behaviour, strategy)
    }

    fn reschedule(&mut self, behaviour: Box<dyn Behaviour<Message = M>>) {
        self.schedule(behaviour, ScheduleStrategy::End);
    }

    fn remove(&mut self, id: BehaviourId) -> bool {
        self.queue.remove(id)
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

    fn id(&self) -> BehaviourId {
        self.id
    }

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
            id: get_id(),
            kind: ParallelBehaviourImpl(self),
            queue,
        })
    }
}
