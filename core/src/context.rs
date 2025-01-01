use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::behaviour::complex::queue::ScheduleStrategy;
use crate::behaviour::{BehaviourId, BehaviourVec, IntoBehaviour};

pub struct Context<M> {
    pub(crate) container: Option<ContainerContext>,
    pub(crate) agent: Option<AgentContext>,
    pub(crate) local: LocalContext<M>,
}

#[derive(Default)]
pub(crate) struct ContainerContext {
    pub(crate) should_stop: bool,
}

#[derive(Default)]
pub(crate) struct AgentContext {}

pub(crate) struct LocalContext<M> {
    pub(crate) messages: Vec<M>,
    pub(crate) new_behaviours: Option<BTreeMap<ScheduleStrategy, BehaviourVec<M>>>,
    pub(crate) removed_behaviours: Vec<BehaviourId>,
}

impl<M> Default for LocalContext<M> {
    fn default() -> Self {
        Self {
            messages: Vec::with_capacity(0),
            new_behaviours: None,
            removed_behaviours: Vec::with_capacity(0),
        }
    }
}

impl<M> Context<M> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn message_parent(&mut self, message: M) {
        self.local.messages.push(message);
    }

    pub fn stop_container(&mut self) {
        self.container
            .get_or_insert_with(ContainerContext::new)
            .should_stop = true;
    }

    pub fn insert_next_behaviour<K>(&mut self, behaviour: impl IntoBehaviour<K, Message = M>) {
        self.local
            .new_behaviours
            .get_or_insert_with(BTreeMap::default)
            .entry(ScheduleStrategy::Next)
            .or_default()
            .push(behaviour.into_behaviour());
    }

    pub fn append_behaviour<K>(&mut self, behaviour: impl IntoBehaviour<K, Message = M>) {
        self.local
            .new_behaviours
            .get_or_insert_with(BTreeMap::default)
            .entry(ScheduleStrategy::End)
            .or_default()
            .push(behaviour.into_behaviour());
    }

    pub fn remove_behaviour(&mut self, id: BehaviourId) {
        self.local.removed_behaviours.push(id);
    }
}

impl<M> Context<M> {
    pub(crate) fn merge<M2>(
        &mut self,
        Context {
            container,
            agent,
            local: _,
        }: Context<M2>,
    ) {
        if let Some(container) = container {
            self.container
                .get_or_insert_with(ContainerContext::new)
                .merge(container);
        }
        if let Some(agent) = agent {
            self.agent
                .get_or_insert_with(AgentContext::new)
                .merge(agent);
        }
    }
}

impl ContainerContext {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn merge(&mut self, Self { should_stop }: Self) {
        self.should_stop |= should_stop;
    }
}

impl AgentContext {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn merge(&mut self, _other: Self) {}
}

impl<M> Default for Context<M> {
    fn default() -> Self {
        Self {
            container: None,
            agent: None,
            local: LocalContext::default(),
        }
    }
}
