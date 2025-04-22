use alloc::collections::btree_map::BTreeMap;
use alloc::vec::Vec;
use core::ops::{Deref, DerefMut};

use crate::acl::message::MessageEnvelope;
use crate::agent::{Aid, AmsAgent};
use crate::container::AgentLike;

#[derive(Debug, Clone)]
pub(crate) struct AgentReference {
    pub(crate) inbox: Vec<MessageEnvelope>,
}

#[derive(Debug, Clone)]
pub(crate) struct Adt(BTreeMap<Aid, AgentReference>);

impl Deref for Adt {
    type Target = BTreeMap<Aid, AgentReference>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Adt {
    pub(super) fn new(ams: &AmsAgent) -> Self {
        Self(BTreeMap::from([(
            ams.get_aid(),
            AgentReference { inbox: Vec::new() },
        )]))
    }
}

impl DerefMut for Adt {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
