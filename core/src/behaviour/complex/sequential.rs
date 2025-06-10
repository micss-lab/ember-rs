use alloc::boxed::Box;
use alloc::collections::vec_deque::VecDeque;

use super::blocked::BlockTracker;
use super::scheduler::BehaviourScheduler;
use super::{
    get_id, Behaviour, BehaviourId, ComplexBehaviour, ComplexBehaviourImpl, Context, IntoBehaviour,
};

pub trait SequentialBehaviour: ComplexBehaviour {
    fn initial_behaviours(
        &self,
    ) -> impl IntoIterator<Item = Box<dyn Behaviour<Event = Self::ChildEvent>>>;
}

pub struct SequentialBehaviourQueue<E> {
    blocked: BlockTracker,
    behaviours: VecDeque<Box<dyn Behaviour<Event = E>>>,
}

impl<E: 'static> SequentialBehaviourQueue<E> {
    pub fn new<K>(behaviours: impl IntoIterator<Item = impl IntoBehaviour<K, Event = E>>) -> Self {
        let behaviours: VecDeque<_> = behaviours.into_iter().map(|b| b.into_behaviour()).collect();
        let blocked = BlockTracker::new(behaviours.iter().map(|b| b.id()));
        Self {
            blocked,
            behaviours,
        }
    }
}

impl<E: 'static> BehaviourScheduler<E> for SequentialBehaviourQueue<E> {
    fn next(&mut self) -> Option<Box<dyn Behaviour<Event = E>>> {
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

    fn reschedule(&mut self, behaviour: Box<dyn Behaviour<Event = E>>) {
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

#[repr(transparent)]
struct SequentialBehaviourImpl<S>(S);

impl<S: SequentialBehaviour> ComplexBehaviour for SequentialBehaviourImpl<S> {
    type Event = S::Event;

    type ChildEvent = S::ChildEvent;

    fn handle_child_event(&mut self, message: Self::ChildEvent) {
        self.0.handle_child_event(message)
    }

    fn after_child_action(&mut self, ctx: &mut Context<Self::Event>) {
        self.0.after_child_action(ctx)
    }
}

#[doc(hidden)]
pub struct Sequential;

impl<T, E: 'static> IntoBehaviour<Sequential> for T
where
    T: SequentialBehaviour<Event = E> + 'static,
{
    type Event = E;

    fn into_behaviour(self) -> Box<dyn Behaviour<Event = Self::Event>> {
        let queue = SequentialBehaviourQueue::new(self.initial_behaviours());
        Box::new(ComplexBehaviourImpl {
            id: get_id(),
            user_impl: SequentialBehaviourImpl(self),
            scheduler: queue,
        })
    }
}
