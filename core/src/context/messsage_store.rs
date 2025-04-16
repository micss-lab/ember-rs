use alloc::{borrow::Cow, collections::vec_deque::VecDeque};

use crate::acl::message::{Message, MessageFilter};

#[derive(Default)]
pub(crate) struct MessageStore {
    messages: VecDeque<Message>,
}

impl MessageStore {
    /// Create a new store from an existing one by moving all the messages.
    pub(super) fn take(&mut self) -> Self {
        Self {
            messages: core::mem::take(&mut self.messages),
        }
    }

    /// Find the first message that matches the filter and remove it from the store.
    pub(super) fn find_and_take(&mut self, filter: Option<&MessageFilter>) -> Option<Message> {
        log::trace!(
            "Trying to find a message matching the filter among {} messages",
            self.messages.len()
        );
        let filter = filter.map_or_else(|| Cow::Owned(MessageFilter::all()), Cow::Borrowed);
        self.messages
            .iter()
            .position(|m| filter.matches(m))
            .map(|p| {
                self.messages
                    .remove(p)
                    .expect("message should be in the list")
            })
    }

    pub(super) fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}

impl FromIterator<Message> for MessageStore {
    fn from_iter<T: IntoIterator<Item = Message>>(iter: T) -> Self {
        Self {
            messages: iter.into_iter().collect(),
        }
    }
}

impl<V> From<V> for MessageStore
where
    V: Into<VecDeque<Message>>,
{
    fn from(value: V) -> Self {
        Self {
            messages: value.into(),
        }
    }
}
