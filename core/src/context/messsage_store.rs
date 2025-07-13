use alloc::{borrow::Cow, collections::vec_deque::VecDeque};

use crate::acl::message::{Message, MessageEnvelope, MessageFilter};

#[derive(Default, Debug, Clone)]
pub(crate) struct MessageStore {
    messages: VecDeque<MessageEnvelope>,
}

impl MessageStore {
    /// Create a new store from an existing one by moving all the messages.
    pub(super) fn take(&mut self) -> Self {
        Self {
            messages: core::mem::take(&mut self.messages),
        }
    }

    /// Find the first message that matches the filter and remove it from the store.
    pub(super) fn find_and_take_as_message(
        &mut self,
        filter: Option<Cow<'_, MessageFilter>>,
    ) -> Option<Message> {
        use crate::acl::message::MessageKind;
        log::trace!(
            "Trying to find a message matching the filter among {} messages",
            self.messages.len()
        );
        let filter = filter.unwrap_or_else(|| Cow::Owned(MessageFilter::all()));
        self.messages
            .iter()
            .map(|m| match m.message {
                MessageKind::Structured(ref m) => m,
            })
            .position(|m| filter.matches(m))
            .map(|p| {
                self.messages
                    .remove(p)
                    .expect("message should be in the list")
            })
            .map(|m| match m.message {
                MessageKind::Structured(m) => m,
            })
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}

impl FromIterator<MessageEnvelope> for MessageStore {
    fn from_iter<T: IntoIterator<Item = MessageEnvelope>>(iter: T) -> Self {
        Self {
            messages: iter.into_iter().collect(),
        }
    }
}

impl IntoIterator for MessageStore {
    type Item = MessageEnvelope;

    type IntoIter = <VecDeque<MessageEnvelope> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.messages.into_iter()
    }
}

impl<V> From<V> for MessageStore
where
    V: Into<VecDeque<MessageEnvelope>>,
{
    fn from(value: V) -> Self {
        Self {
            messages: value.into(),
        }
    }
}
