use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::message::filter::MessageFilter;
use crate::message::{Message, TransportMessage};

pub use self::messsage_store::MessageStore;

mod messsage_store;

#[derive(Default)]
pub struct Environment {
    pub stop_platform: bool,
    pub message_outbox: Vec<(TransportMessage, SendCallbacks)>,
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

    pub fn send_message(&mut self, message: Message) -> SendCallbackBuilder<'_> {
        self.message_outbox
            .push((message.into_transport(), SendCallbacks::default()));
        let (_, callbacks) = self
            .message_outbox
            .last_mut()
            .expect("message_outbox should have the just-pushed message as its last element");
        SendCallbackBuilder { callbacks }
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

type OnceCallback = Box<dyn FnOnce(&mut Environment)>;
type RetryCallback = Box<dyn FnMut(u32, &mut Environment)>;

#[derive(Default)]
pub struct SendCallbacks {
    pub on_success: Option<OnceCallback>,
    pub on_retry: Option<RetryCallback>,
    pub on_failure: Option<OnceCallback>,
    /// Fires alongside final outcomes (success or failure).
    pub on_complete: Option<OnceCallback>,
}

impl SendCallbacks {
    pub fn on_success(mut self, f: impl FnOnce(&mut Environment) + 'static) -> Self {
        self.on_success = Some(Box::new(f));
        self
    }

    pub fn on_retry(mut self, f: impl FnMut(u32, &mut Environment) + 'static) -> Self {
        self.on_retry = Some(Box::new(f));
        self
    }

    pub fn on_failure(mut self, f: impl FnOnce(&mut Environment) + 'static) -> Self {
        self.on_failure = Some(Box::new(f));
        self
    }

    pub fn on_complete(mut self, f: impl FnOnce(&mut Environment) + 'static) -> Self {
        self.on_complete = Some(Box::new(f));
        self
    }
}

pub struct SendCallbackBuilder<'e> {
    callbacks: &'e mut SendCallbacks,
}

impl SendCallbackBuilder<'_> {
    pub fn on_success(self, f: impl FnOnce(&mut Environment) + 'static) -> Self {
        self.callbacks.on_success = Some(Box::new(f));
        self
    }

    pub fn on_retry(self, f: impl FnMut(u32, &mut Environment) + 'static) -> Self {
        self.callbacks.on_retry = Some(Box::new(f));
        self
    }

    pub fn on_failure(self, f: impl FnOnce(&mut Environment) + 'static) -> Self {
        self.callbacks.on_failure = Some(Box::new(f));
        self
    }

    pub fn on_complete(self, f: impl FnOnce(&mut Environment) + 'static) -> Self {
        self.callbacks.on_complete = Some(Box::new(f));
        self
    }
}
