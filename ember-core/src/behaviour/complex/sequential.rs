use alloc::boxed::Box;
use alloc::collections::vec_deque::VecDeque;

use super::blocked::BlockTracker;
use super::scheduler::BehaviourScheduler;
use super::{
    Behaviour, BehaviourId, ComplexBehaviour, ComplexBehaviourImpl, Context, IntoBehaviour,
    ScheduledComplexBehaviour, get_id,
};

pub trait SequentialBehaviour<'a>: ComplexBehaviour
where
    Self: 'a,
{
    fn initial_behaviours(
        &self,
    ) -> impl IntoIterator<
        Item = Box<dyn Behaviour<AgentState = Self::AgentState, Event = Self::ChildEvent> + 'a>,
    >;
}

pub struct SequentialBehaviourQueue<'a, S, E> {
    blocked: BlockTracker,
    behaviours: VecDeque<Box<dyn Behaviour<AgentState = S, Event = E> + 'a>>,
}

impl<'a, S, E> SequentialBehaviourQueue<'a, S, E> {
    pub fn new<K>(
        behaviours: impl IntoIterator<Item = impl IntoBehaviour<'a, K, AgentState = S, Event = E>>,
    ) -> Self {
        let behaviours: VecDeque<_> = behaviours.into_iter().map(|b| b.into_behaviour()).collect();
        let blocked = BlockTracker::new(behaviours.iter().map(|b| b.id()));
        Self {
            blocked,
            behaviours,
        }
    }
}

impl<'a, S, E> BehaviourScheduler<'a, S, E> for SequentialBehaviourQueue<'a, S, E> {
    fn next(&mut self) -> Option<Box<dyn Behaviour<AgentState = S, Event = E> + 'a>> {
        let behaviour = self.behaviours.pop_front()?;
        let id = behaviour.id();
        if self
            .blocked
            .is_blocked(id)
            .expect("scheduled behaviour should be registered with block tracker")
        {
            self.reschedule(behaviour);
            return None;
        }
        self.blocked.unregister(id);
        Some(behaviour)
    }

    fn reschedule(&mut self, behaviour: Box<dyn Behaviour<AgentState = S, Event = E> + 'a>) {
        self.blocked.register(behaviour.id());
        self.behaviours.push_front(behaviour);
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
        self.behaviours.is_empty()
    }
}

struct SequentialBehaviourImpl<'a, S: SequentialBehaviour<'a>> {
    user_impl: S,
    queue: SequentialBehaviourQueue<'a, S::AgentState, S::ChildEvent>,
}

impl<'a, S: SequentialBehaviour<'a>> ComplexBehaviour for SequentialBehaviourImpl<'a, S> {
    type AgentState = S::AgentState;

    type Event = S::Event;

    type ChildEvent = S::ChildEvent;

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

impl<'a, S: SequentialBehaviour<'a>> ScheduledComplexBehaviour<'a>
    for SequentialBehaviourImpl<'a, S>
{
    fn scheduler(
        &mut self,
    ) -> &mut impl BehaviourScheduler<'a, Self::AgentState, Self::ChildEvent> {
        &mut self.queue
    }
}

#[doc(hidden)]
pub struct Sequential;

impl<'a, T, S, E> IntoBehaviour<'a, Sequential> for T
where
    T: SequentialBehaviour<'a, AgentState = S, Event = E>,
{
    type AgentState = S;

    type Event = E;

    fn into_behaviour(
        self,
    ) -> Box<dyn Behaviour<AgentState = Self::AgentState, Event = Self::Event> + 'a> {
        let queue = SequentialBehaviourQueue::new(self.initial_behaviours());
        Box::new(ComplexBehaviourImpl {
            id: get_id(),
            inner: SequentialBehaviourImpl {
                user_impl: self,
                queue,
            },
        })
    }
}
