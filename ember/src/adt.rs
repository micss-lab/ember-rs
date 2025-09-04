use alloc::collections::btree_map::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::ops::{Deref, DerefMut};

use crate::agent::AmsAgent;
use ember_core::agent::Agent;
use ember_core::agent::aid::Aid;
use ember_core::context::MessageStore;
use ember_core::message::MessageEnvelope;

#[derive(Debug, Clone)]
pub(crate) enum AgentReference {
    Local(LocalAgentReference),
    Proxy(Aid),
}

#[derive(Debug, Clone)]
pub(crate) struct LocalAgentReference {
    pub(crate) inbox: Vec<MessageEnvelope>,
}

#[derive(Debug, Clone)]
pub(crate) struct Adt(BTreeMap<String, AgentReference>);

impl Deref for Adt {
    type Target = BTreeMap<String, AgentReference>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Adt {
    pub(super) fn new(ams: &AmsAgent) -> Self {
        Self(BTreeMap::from([(
            ams.get_name().to_string(),
            AgentReference::Local(LocalAgentReference { inbox: Vec::new() }),
        )]))
    }

    pub(crate) fn agent_has_message(&self, agent_name: impl AsRef<str>) -> bool {
        self.get_local(agent_name.as_ref())
            .is_some_and(|r| !r.inbox.is_empty())
    }

    pub(crate) fn messages_for_agent(
        &mut self,
        agent_name: impl AsRef<str>,
    ) -> Option<Vec<MessageEnvelope>> {
        Some(
            core::mem::take(&mut self.get_local_mut(agent_name.as_ref())?.inbox)
                .into_iter()
                .collect(),
        )
    }

    pub(crate) fn return_unhandled_messages(
        &mut self,
        agent_name: impl AsRef<str>,
        messages: MessageStore,
    ) {
        if messages.is_empty() {
            return;
        }
        self.get_local_mut(agent_name)
            .expect("agent returning messages should be in ladt")
            .inbox
            .extend(messages);
    }

    fn get_local(&self, agent_name: impl AsRef<str>) -> Option<&LocalAgentReference> {
        let AgentReference::Local(local) = self.get(agent_name.as_ref())? else {
            panic!("agent {} is not local", agent_name.as_ref());
        };
        Some(local)
    }

    fn get_local_mut(&mut self, agent_name: impl AsRef<str>) -> Option<&mut LocalAgentReference> {
        let AgentReference::Local(local) = self.get_mut(agent_name.as_ref())? else {
            panic!("agent {} is not local", agent_name.as_ref());
        };
        Some(local)
    }
}

impl DerefMut for Adt {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
