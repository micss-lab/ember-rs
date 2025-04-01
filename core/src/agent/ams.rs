use alloc::borrow::Cow;
use alloc::collections::btree_map::BTreeMap;
use alloc::string::String;

use uchan::Sender;

use crate::acl::message::MessageEnvelope;
use crate::behaviour::OneShotBehaviour;
use crate::container::AgentLike;
use crate::context::{ContainerContext, Context};

use super::Agent;

type Aid = String;

pub(crate) struct AmsAgent {
    /// Inner agent on which ams behaviours will be stored.
    inner: Agent<()>,
    ladt: BTreeMap<Aid, AgentReference>,
}

impl AgentLike for AmsAgent {
    fn update(&mut self, context: &mut ContainerContext) -> bool {
        self.inner.update(context)
    }

    fn get_name(&self) -> Cow<str> {
        self.inner.get_name()
    }
}

impl AmsAgent {
    pub(crate) fn new() -> Self {
        let inner = Agent::new("ams").with_behaviour(StartupMessage);
        Self {
            inner,
            ladt: BTreeMap::new(),
        }
    }
}

struct AgentReference {
    postbus: Sender<MessageEnvelope>,
}

struct StartupMessage;

impl OneShotBehaviour for StartupMessage {
    type Event = ();

    fn action(&self, _: &mut Context<Self::Event>) {
        log::debug!("Ams agent has registered.");
    }
}
