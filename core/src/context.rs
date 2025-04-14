use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use self::messsage_store::MessageStore;
use crate::acl::message::{Message, MessageEnvelope, MessageFilter};
use crate::behaviour::complex::queue::ScheduleStrategy;
use crate::behaviour::{BehaviourId, BehaviourVec, IntoBehaviour};

mod messsage_store;

pub struct Context<E> {
    pub(crate) container: Option<ContainerContext>,
    pub(crate) agent: AgentContext,
    pub(crate) local: LocalContext<E>,
}

#[derive(Default)]
pub(crate) struct ContainerContext {
    pub(crate) should_stop: bool,
}

#[derive(Default)]
pub(crate) struct AgentContext {
    pub(crate) should_remove: bool,
    pub(crate) message_inbox: MessageStore,
    pub(crate) message_to_send: Vec<MessageEnvelope>,
}

pub(crate) struct LocalContext<E> {
    pub(crate) events: Vec<E>,
    pub(crate) new_behaviours: Option<BTreeMap<ScheduleStrategy, BehaviourVec<E>>>,
    pub(crate) removed_behaviours: Vec<BehaviourId>,
    pub(crate) should_block: bool,
}

impl<E> Default for LocalContext<E> {
    fn default() -> Self {
        Self {
            events: Vec::with_capacity(0),
            new_behaviours: None,
            removed_behaviours: Vec::with_capacity(0),
            should_block: false,
        }
    }
}

impl<E: 'static> Context<E> {
    pub(crate) fn new(messages: impl Into<MessageStore>) -> Self {
        Self {
            agent: AgentContext::new(messages),
            ..Default::default()
        }
    }

    pub(crate) fn from_upper<M2>(upper: &mut Context<M2>) -> Self {
        Self::new(upper.agent.message_inbox.drain_into_new())
    }

    pub fn emit_event(&mut self, event: E) {
        self.local.events.push(event);
    }

    pub fn stop_container(&mut self) {
        self.container
            .get_or_insert_with(ContainerContext::new)
            .should_stop = true;
    }

    pub fn remove_agent(&mut self) {
        self.agent.should_remove = true;
    }

    pub fn block_behaviour(&mut self) {
        self.local.should_block = true;
    }

    fn insert_behaviour<K>(
        &mut self,
        behaviour: impl IntoBehaviour<K, Event = E>,
        strategy: ScheduleStrategy,
    ) -> BehaviourId {
        let behaviour = behaviour.into_behaviour();
        let id = behaviour.id();
        self.local
            .new_behaviours
            .get_or_insert_with(BTreeMap::default)
            .entry(strategy)
            .or_default()
            .push(behaviour);
        id
    }

    pub fn insert_next_behaviour<K>(
        &mut self,
        behaviour: impl IntoBehaviour<K, Event = E>,
    ) -> BehaviourId {
        self.insert_behaviour(behaviour, ScheduleStrategy::Next)
    }

    pub fn append_behaviour<K>(
        &mut self,
        behaviour: impl IntoBehaviour<K, Event = E>,
    ) -> BehaviourId {
        self.insert_behaviour(behaviour, ScheduleStrategy::End)
    }

    pub fn remove_behaviour(&mut self, id: BehaviourId) {
        self.local.removed_behaviours.push(id);
    }

    pub fn receive_message(&mut self, filter: Option<&MessageFilter>) -> Option<Message> {
        self.agent.message_inbox.find_and_take(filter)
    }
}

impl<E> Context<E> {
    pub(crate) fn merge<E2>(
        &mut self,
        Context {
            container,
            agent,
            local: _,
        }: Context<E2>,
    ) {
        if let Some(container) = container {
            self.container
                .get_or_insert_with(ContainerContext::new)
                .merge(container);
        }
        self.agent.merge(agent);
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
    pub(crate) fn new(messages: impl Into<MessageStore>) -> Self {
        let messages = messages.into();
        Self {
            should_remove: false,
            message_inbox: messages,
            message_to_send: Vec::new(),
        }
    }

    pub(crate) fn merge(&mut self, other: Self) {
        // NOTE: Messages inboxes should always be passed down completely
        assert!(
            self.message_inbox.is_empty(),
            "message inboxe should be passed down fully"
        );
        self.message_inbox = other.message_inbox;
        self.message_to_send.extend(other.message_to_send);
    }
}

impl<E> Default for Context<E> {
    fn default() -> Self {
        Self {
            container: None,
            agent: AgentContext::default(),
            local: LocalContext::default(),
        }
    }
}
