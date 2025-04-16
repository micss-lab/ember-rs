use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use self::messsage_store::MessageStore;
use crate::acl::message::{Message, MessageEnvelope, MessageFilter};
use crate::behaviour::complex::queue::ScheduleStrategy;
use crate::behaviour::{BehaviourId, BehaviourVec, IntoBehaviour};

mod messsage_store;

pub struct Context<E> {
    pub(crate) container: ContainerContext,
    pub(crate) agent: AgentContext,
    pub(crate) local: LocalContext<E>,
}

#[derive(Default)]
pub(crate) struct ContainerContext {
    pub(crate) should_stop: bool,
    pub(crate) message_outbox: Vec<MessageEnvelope>,
    pub(crate) message_inbox: MessageStore,
}

#[derive(Default)]
pub(crate) struct AgentContext {
    pub(crate) should_remove: bool,
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
    pub(crate) fn new_using_container(container_ctx: &mut ContainerContext) -> Self {
        Self {
            container: ContainerContext::new(container_ctx.message_inbox.take()),
            ..Default::default()
        }
    }

    pub(crate) fn from_upper<E2>(upper: &mut Context<E2>) -> Self {
        Self::new_using_container(&mut upper.container)
    }

    pub fn emit_event(&mut self, event: E) {
        self.local.events.push(event);
    }

    pub fn stop_container(&mut self) {
        self.container.should_stop = true;
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
        self.container.message_inbox.find_and_take(filter)
    }

    pub fn send_message(&mut self, message: MessageEnvelope) {
        self.container.send_message(message);
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
        self.container.merge(container);
        self.agent.merge(agent);
    }
}

impl ContainerContext {
    pub(crate) fn new(messages: impl Into<MessageStore>) -> Self {
        let messages = messages.into();
        Self {
            message_inbox: messages,
            ..Default::default()
        }
    }

    pub(crate) fn merge(
        &mut self,
        Self {
            should_stop,
            message_outbox,
            message_inbox,
        }: Self,
    ) {
        self.should_stop |= should_stop;
        debug_assert!(
            self.message_outbox.is_empty(),
            "container level context should not have any messages in the outbox"
        );
        debug_assert!(
            self.message_inbox.is_empty(),
            "message inbox should have been passed down fully"
        );
        self.message_outbox = message_outbox;
        self.message_inbox = message_inbox;
    }

    pub(crate) fn send_message(&mut self, message: MessageEnvelope) {
        self.message_outbox.push(message)
    }
}

impl AgentContext {
    fn merge(&mut self, other: Self) {
        self.should_remove |= other.should_remove;
    }
}

impl<E> Default for Context<E> {
    fn default() -> Self {
        Self {
            container: ContainerContext::default(),
            agent: AgentContext::default(),
            local: LocalContext::default(),
        }
    }
}
