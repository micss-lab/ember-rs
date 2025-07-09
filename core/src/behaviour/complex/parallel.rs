use alloc::boxed::Box;
use alloc::collections::{BTreeSet, VecDeque};

use super::blocked::BlockTracker;
use super::scheduler::BehaviourScheduler;
use super::{
    get_id, Behaviour, BehaviourId, ComplexBehaviour, ComplexBehaviourImpl, Context, IntoBehaviour,
    ScheduledComplexBehaviour,
};

pub trait ParallelBehaviour: ComplexBehaviour {
    fn finish_strategy(&self) -> FinishStrategy;

    fn initial_behaviours(
        &self,
    ) -> impl IntoIterator<
        Item = Box<dyn Behaviour<AgentState = Self::AgentState, Event = Self::ChildEvent>>,
    >;
}

pub(crate) struct ParallelBehaviourQueue<S, E> {
    blocked: BlockTracker,
    behaviours: VecDeque<Box<dyn Behaviour<AgentState = S, Event = E>>>,
    finished: usize,
    strategy: FinishStrategy,
}

pub enum FinishStrategy {
    All,
    One,
    N(usize),
    Never,
}

impl<S: 'static, E: 'static> ParallelBehaviourQueue<S, E> {
    pub fn new<K>(
        behaviours: impl IntoIterator<Item = impl IntoBehaviour<K, AgentState = S, Event = E>>,
        strategy: FinishStrategy,
    ) -> Self {
        let behaviours: VecDeque<_> = behaviours.into_iter().map(|b| b.into_behaviour()).collect();
        let blocked = BlockTracker::new(behaviours.iter().map(|b| b.id()));
        Self {
            blocked,
            behaviours,
            finished: 0,
            strategy,
        }
    }

    pub(crate) fn new_empty(strategy: FinishStrategy) -> Self {
        Self {
            blocked: BlockTracker::default(),
            behaviours: VecDeque::default(),
            finished: 0,
            strategy,
        }
    }

    pub(crate) fn with_behaviour<K>(
        mut self,
        behaviour: impl IntoBehaviour<K, AgentState = S, Event = E>,
    ) -> Self {
        self.add_behaviour(behaviour);
        self
    }

    pub(crate) fn add_behaviour<K>(
        &mut self,
        behaviour: impl IntoBehaviour<K, AgentState = S, Event = E>,
    ) {
        let behaviour = behaviour.into_behaviour();
        self.blocked.register(behaviour.id());
        self.behaviours.push_back(behaviour);
    }
}

impl<S: 'static, E: 'static> BehaviourScheduler<S, E> for ParallelBehaviourQueue<S, E> {
    fn next(&mut self) -> Option<Box<dyn Behaviour<AgentState = S, Event = E>>> {
        let mut seen = BTreeSet::new();
        while let Some(behaviour) = self.behaviours.pop_front() {
            let id = behaviour.id();

            if self
                .blocked
                .is_blocked(id)
                .expect("scheduled behaviour should be registered with block tracker")
            {
                self.reschedule(behaviour);
                if !seen.insert(id) {
                    return None;
                }
                continue;
            }

            self.blocked.unregister(id);
            return Some(behaviour);
        }
        None
    }

    fn reschedule(&mut self, behaviour: Box<dyn Behaviour<AgentState = S, Event = E>>) {
        self.blocked.register(behaviour.id());
        self.behaviours.push_back(behaviour);
    }

    fn remove(&mut self, id: BehaviourId) -> bool {
        let len = self.behaviours.len();
        self.blocked.unregister(id);
        self.behaviours.retain(|b| b.id() != id);
        len != self.behaviours.len()
    }

    fn block(&mut self, id: BehaviourId) -> bool {
        self.blocked.block(id)
    }

    fn unblock_all(&mut self) {
        self.blocked.unblock_all();
    }

    fn is_finished(&self) -> bool {
        match self.strategy {
            FinishStrategy::All => self.behaviours.is_empty(),
            FinishStrategy::One => self.finished >= 1,
            FinishStrategy::N(n) => self.finished >= n,
            FinishStrategy::Never => false,
        }
    }
}

struct ParallelBehaviourImpl<P: ParallelBehaviour> {
    user_impl: P,
    queue: ParallelBehaviourQueue<P::AgentState, P::ChildEvent>,
}

impl<P: ParallelBehaviour> ComplexBehaviour for ParallelBehaviourImpl<P> {
    type Event = P::Event;

    type ChildEvent = P::ChildEvent;

    type AgentState = P::AgentState;

    fn handle_child_event(&mut self, event: Self::ChildEvent) {
        self.user_impl.handle_child_event(event)
    }

    fn after_child_action(
        &mut self,
        ctx: &mut Context<Self::Event>,
        agent_state: &mut Self::AgentState,
    ) {
        self.user_impl.after_child_action(ctx, agent_state)
    }
}

impl<P: ParallelBehaviour> ScheduledComplexBehaviour for ParallelBehaviourImpl<P>
where
    Self::AgentState: 'static,
    Self::ChildEvent: 'static,
{
    fn scheduler(&mut self) -> &mut impl BehaviourScheduler<Self::AgentState, Self::ChildEvent> {
        &mut self.queue
    }
}

#[doc(hidden)]
pub struct Parallel;

impl<T: 'static, S: 'static, E: 'static> IntoBehaviour<Parallel> for T
where
    T: ParallelBehaviour<AgentState = S, Event = E>,
{
    type Event = E;

    type AgentState = S;

    fn into_behaviour(self) -> Box<dyn Behaviour<AgentState = S, Event = Self::Event>> {
        let queue = ParallelBehaviourQueue::new(self.initial_behaviours(), self.finish_strategy());
        Box::new(ComplexBehaviourImpl {
            id: get_id(),
            inner: ParallelBehaviourImpl {
                user_impl: self,
                queue,
            },
        })
    }
}
