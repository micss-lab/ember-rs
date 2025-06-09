use alloc::boxed::Box;
use alloc::collections::vec_deque::VecDeque;

use super::blocked::BlockTracker;
use super::macros::{complex_action_impl, complex_behaviour_methods};
use super::scheduler::BehaviourScheduler;
use super::{get_id, Behaviour, BehaviourId, ComplexBehaviour, Context, IntoBehaviour};

pub trait SequentialBehaviour {
    type Event;

    type ChildEvent;

    fn initial_behaviours(
        &self,
    ) -> impl IntoIterator<Item = Box<dyn Behaviour<Event = Self::ChildEvent>>>;

    complex_behaviour_methods!();
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

struct SequentialBehaviourImpl<S: SequentialBehaviour>(S);

impl<S, E: 'static, CE: 'static> Behaviour
    for ComplexBehaviour<SequentialBehaviourImpl<S>, SequentialBehaviourQueue<CE>>
where
    S: SequentialBehaviour<Event = E, ChildEvent = CE> + 'static,
{
    type Event = E;

    fn id(&self) -> BehaviourId {
        self.id
    }

    complex_action_impl!();
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
        Box::new(ComplexBehaviour {
            id: get_id(),
            kind: SequentialBehaviourImpl(self),
            queue,
        })
    }
}
