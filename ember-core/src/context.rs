use alloc::borrow::Cow;
use alloc::vec::Vec;

use crate::behaviour::BehaviourId;
use crate::message::filter::MessageFilter;
use crate::message::{Message, MessageEnvelope};

pub use self::messsage_store::MessageStore;

mod messsage_store;

pub struct Context<E> {
    pub container: ContainerContext,
    pub agent: AgentContext,
    pub local: LocalContext<E>,
}

#[derive(Default)]
pub struct ContainerContext {
    pub should_stop: bool,
    pub message_outbox: Vec<MessageEnvelope>,
    pub message_inbox: MessageStore,
    pub new_messages: bool,
}

#[derive(Default)]
pub struct AgentContext {
    pub should_remove: bool,
}

pub struct LocalContext<E> {
    pub events: Vec<E>,
    pub removed_behaviours: Vec<BehaviourId>,
    pub block: bool,
    pub reset: bool,
}

impl<E> Default for LocalContext<E> {
    fn default() -> Self {
        Self {
            events: Vec::with_capacity(0),
            removed_behaviours: Vec::with_capacity(0),
            block: false,
            reset: false,
        }
    }
}

impl<E: 'static> Context<E> {
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
        self.local.block = true;
    }

    pub fn reset_behaviour(&mut self) {
        self.local.reset = true;
    }

    pub fn remove_behaviour(&mut self, id: BehaviourId) {
        self.local.removed_behaviours.push(id);
    }

    pub fn receive_message(&mut self, filter: Option<Cow<'_, MessageFilter>>) -> Option<Message> {
        self.container
            .message_inbox
            .find_and_take_as_message(filter)
    }

    pub fn send_message(&mut self, message: MessageEnvelope) {
        self.container.send_message(message);
    }
}

impl<E: 'static> Context<E> {
    pub fn new_using_container(container_ctx: &mut ContainerContext) -> Self {
        Self {
            container: ContainerContext::new(container_ctx.message_inbox.take()),
            ..Default::default()
        }
    }

    pub fn from_upper<E2>(upper: &mut Context<E2>) -> Self {
        Self::new_using_container(&mut upper.container)
    }
}

impl<E> Context<E> {
    pub fn merge<E2>(
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
    pub fn new(messages: impl Into<MessageStore>) -> Self {
        let messages = messages.into();
        let new_messages = !messages.is_empty();
        Self {
            message_inbox: messages,
            new_messages,
            ..Default::default()
        }
    }

    pub fn merge(
        &mut self,
        Self {
            should_stop,
            message_outbox,
            message_inbox,
            new_messages,
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
        self.new_messages = new_messages;
    }

    pub fn send_message(&mut self, message: MessageEnvelope) {
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
