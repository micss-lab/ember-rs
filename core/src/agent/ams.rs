use alloc::borrow::Cow;

use crate::behaviour::OneShotBehaviour;
use crate::container::AgentLike;
use crate::context::{ContainerContext, Context};

use super::Agent;

pub(crate) struct AmsAgent {
    /// Inner agent on which all behaviours will be stored.
    inner: Agent<()>,
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
        Self { inner }
    }
}

struct StartupMessage;

impl OneShotBehaviour for StartupMessage {
    type Message = ();

    fn action(&self, ctx: &mut Context<Self::Message>) {
        log::info!("Ams agent has registered.");
    }
}
