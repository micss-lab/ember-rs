use alloc::borrow::Cow;
use alloc::vec::Vec;

use crate::message::filter::MessageFilter;
use crate::message::{Message, TransportMessage};

pub use self::messsage_store::MessageStore;

mod messsage_store;

#[derive(Clone, Default)]
pub struct Environment {
    pub stop_platform: bool,
    pub message_outbox: Vec<TransportMessage>,
    pub message_inbox: MessageStore,
    pub new_messages: bool,
}

impl Environment {
    pub fn stop_platform(&mut self) {
        self.stop_platform = true;
    }

    pub fn receive_message(&mut self, filter: Option<Cow<'_, MessageFilter>>) -> Option<Message> {
        self.message_inbox.find_and_take(filter)
    }

    pub fn send_message(&mut self, message: Message) {
        self.message_outbox.push(message.into_transport())
    }
}

impl Environment {
    pub fn new(messages: impl Into<MessageStore>) -> Self {
        let messages = messages.into();
        let new_messages = !messages.is_empty();
        Self {
            message_inbox: messages,
            new_messages,
            ..Default::default()
        }
    }
}
