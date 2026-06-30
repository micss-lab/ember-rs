use alloc::borrow::Cow;
use alloc::collections::VecDeque;

use crate::message::Message;
use crate::message::filter::MessageFilter;

#[derive(Default, Debug, Clone)]
pub struct MessageStore {
    messages: VecDeque<Message>,
}

impl MessageStore {
    /// Create a new store from an existing one by moving all the messages.
    pub fn take(&mut self) -> Self {
        Self {
            messages: core::mem::take(&mut self.messages),
        }
    }

    /// Find the first message that matches the filter and remove it from the store.
    pub fn find_and_take(&mut self, filter: Option<Cow<'_, MessageFilter>>) -> Option<Message> {
        let filter = filter.unwrap_or_else(|| Cow::Owned(MessageFilter::all()));
        self.messages
            .iter()
            .position(|m| filter.matches(m))
            .map(|p| {
                self.messages
                    .remove(p)
                    .expect("message should be in the list")
            })
    }

    pub fn is_empty(&self) -> bool {
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

impl IntoIterator for MessageStore {
    type Item = Message;

    type IntoIter = <VecDeque<Message> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.messages.into_iter()
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
