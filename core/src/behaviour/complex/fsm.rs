use alloc::boxed::Box;
use alloc::collections::btree_map::BTreeMap;
use alloc::collections::BTreeSet;

use super::blocked::BlockTracker;
use super::scheduler::BehaviourScheduler;
use super::{
    get_id, Behaviour, BehaviourId, ComplexBehaviour, ComplexBehaviourImpl, Context, IntoBehaviour,
};

pub trait FsmBehaviour: ComplexBehaviour {
    type TransitionTrigger;

    fn fsm(&self) -> Fsm<Self::TransitionTrigger, Self::ChildEvent>;
}

pub enum FsmEvent<T, E> {
    Trigger(T),
    Event(E),
}

pub struct Fsm<T, E> {
    blocked: BlockTracker,
    current: BehaviourId,
    final_states: BTreeSet<BehaviourId>,
    transitions: BTreeMap<BehaviourId, BTreeMap<T, BehaviourId>>,
    behaviours: BTreeMap<BehaviourId, Box<dyn Behaviour<Event = FsmEvent<T, E>>>>,
}

pub struct FsmBuilder<T, E> {
    final_states: BTreeSet<BehaviourId>,
    transitions: BTreeMap<BehaviourId, BTreeMap<T, BehaviourId>>,
    behaviours: BTreeMap<BehaviourId, Box<dyn Behaviour<Event = FsmEvent<T, E>>>>,
}

impl<T, E> Default for FsmBuilder<T, E> {
    fn default() -> Self {
        Self {
            final_states: BTreeSet::default(),
            transitions: BTreeMap::default(),
            behaviours: BTreeMap::default(),
        }
    }
}

impl<T, E> Fsm<T, E> {
    pub fn builder() -> FsmBuilder<T, E> {
        FsmBuilder::new()
    }
}

impl<T, E> FsmBuilder<T, E> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<T: 'static, E: 'static> FsmBuilder<T, E> {
    pub fn with_behaviour<K>(
        mut self,
        behaviour: impl IntoBehaviour<K, Event = FsmEvent<T, E>>,
        is_final: bool,
    ) -> Self {
        let behaviour = behaviour.into_behaviour();
        let id = behaviour.id();
        self.behaviours.insert(id, behaviour);
        if is_final {
            self.final_states.insert(id);
        }
        self
    }
}

impl<T, E> FsmBuilder<T, E>
where
    T: Ord,
{
    pub fn with_transition(
        mut self,
        src: BehaviourId,
        destination: BehaviourId,
        trigger: T,
    ) -> Self {
        self.transitions
            .entry(src)
            .or_insert_with(BTreeMap::default)
            .insert(trigger, destination);
        self
    }
}

impl<T, E> FsmBuilder<T, E> {
    /// Validates the currently configured [`FsmBuilder`] and returns the fsm based behaviour scheduler.
    // TODO: Return a proper error enum here.
    pub fn try_build(self, start_behaviour: BehaviourId) -> Result<Fsm<T, E>, &'static str> {
        let Self {
            final_states,
            transitions,
            behaviours,
        } = self;
        let ids: BTreeSet<_> = behaviours.keys().copied().collect();

        if !final_states.is_subset(&ids) {
            return Err("Invalid final states.");
        }

        let transition_ids: BTreeSet<_> = {
            let src_ids = transitions.keys().copied();
            let dest_ids = transitions.values().map(|v| v.values()).flatten().copied();
            src_ids.chain(dest_ids).collect()
        };

        if !transition_ids.is_subset(&ids) {
            return Err("Invalid transitions.");
        }

        Ok(Fsm {
            blocked: BlockTracker::new(ids),
            current: start_behaviour,
            final_states,
            transitions,
            behaviours,
        })
    }
}

impl<T: 'static, E: 'static> BehaviourScheduler<FsmEvent<T, E>> for Fsm<T, E> {
    fn next(&mut self) -> Option<Box<dyn Behaviour<Event = FsmEvent<T, E>>>> {
        todo!()
    }

    fn reschedule(&mut self, behaviour: Box<dyn Behaviour<Event = FsmEvent<T, E>>>) {
        todo!()
    }

    fn remove(&mut self, id: BehaviourId) -> bool {
        todo!()
    }

    fn block(&mut self, id: BehaviourId) -> bool {
        self.blocked.block(id)
    }

    fn unblock_all(&mut self) {
        self.blocked.unblock_all();
    }

    fn is_finished(&self) -> bool {
        // Only allow this behaviour to finish if the final behaviour is also finished.
        self.final_states.contains(&self.current) && !self.behaviours.contains_key(&self.current)
    }
}

#[repr(transparent)]
struct FsmBehaviourImpl<F: FsmBehaviour>(F);

impl<F: FsmBehaviour> ComplexBehaviour for FsmBehaviourImpl<F> {
    type Event = F::Event;

    type ChildEvent = FsmEvent<F::TransitionTrigger, F::ChildEvent>;

    fn handle_child_event(&mut self, message: Self::ChildEvent) {
        match message {
            FsmEvent::Trigger(t) => todo!(),
            FsmEvent::Event(e) => self.0.handle_child_event(e),
        }
    }

    fn after_child_action(&mut self, ctx: &mut Context<Self::Event>) {
        self.0.after_child_action(ctx)
    }
}

#[doc(hidden)]
pub struct FsmKind;

impl<T, E: 'static> IntoBehaviour<FsmKind> for T
where
    T: FsmBehaviour<Event = E> + 'static,
{
    type Event = E;

    fn into_behaviour(self) -> Box<dyn Behaviour<Event = Self::Event>> {
        let fsm = self.fsm();
        Box::new(ComplexBehaviourImpl {
            id: get_id(),
            user_impl: FsmBehaviourImpl(self),
            scheduler: fsm,
        })
    }
}
