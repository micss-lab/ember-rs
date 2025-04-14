use alloc::collections::btree_map::BTreeMap;
use core::ops::{Deref, DerefMut};

use uchan::Sender;

use crate::acl::message::MessageEnvelope;
use crate::agent::Aid;

#[derive(Debug, Clone)]
pub(crate) struct AgentReference {
    pub(crate) outbox: Sender<MessageEnvelope>,
    pub(crate) registered: bool,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Adt(BTreeMap<Aid, AgentReference>);

impl Deref for Adt {
    type Target = BTreeMap<Aid, AgentReference>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Adt {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
