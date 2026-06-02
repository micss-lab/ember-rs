use std::borrow::Cow;

use ember_core::agent::{Agent, Aid};
use ember_core::environment::Environment;
use ember_fipa::agent::ams::AmsAgent;

use crate::adt::Adt;

use super::{Container, Mts};

/// Privileged agents able to modify the container/platform directly.
pub(super) trait PrivilegedAgent: Agent {
    fn update_privileged(
        &mut self,
        container: &mut ContainerView<'_, '_>,
        environment: &mut Environment,
    );
}

pub(super) struct ContainerView<'a, 'c> {
    pub(super) ladt: &'a mut Adt,
    pub(super) mts: &'a mut Mts<'c>,
}

#[derive(Default)]
pub(super) struct PrivilegedAgents {
    /// Ams agent managing this cotainers.
    ams: AmsAgent,
}

impl PrivilegedAgents {
    pub(super) fn agent_names(&self) -> impl IntoIterator<Item = Cow<str>> + '_ {
        core::iter::once(self.ams.get_name())
    }

    pub(super) fn poll(&mut self, container: &mut ContainerView<'_, '_>) {
        fn poll_agent(agent: &mut impl PrivilegedAgent, container: &mut ContainerView<'_, '_>) {
            if !container.ladt.agent_has_message(agent.get_name()) {
                // Assume that the agent does not have to be scheduled if there is no message for
                // it available. This might be checked through the trait in the future.
                return;
            }

            let mut environment = Environment::new(
                container
                    .ladt
                    .messages_for_agent(agent.get_name())
                    .unwrap_or_default(),
            );
            agent.update_privileged(container, &mut environment);

            // Handle all messages the agent wants to send.
            for message in environment.message_outbox.into_iter() {
                container.mts.send_message(message, &mut *container.ladt);
            }

            container
                .ladt
                .return_unhandled_messages(agent.get_name(), environment.message_inbox);
        }

        poll_agent(&mut self.ams, &mut *container);
    }
}

mod ams {
    use std::string::ToString;
    use std::vec::Vec;

    use ember_core::agent::Agent;
    use ember_core::agent::aid::Aid;
    use ember_core::environment::{self, Environment};
    use ember_fipa::agent::ams::AmsAgent;
    use ember_fipa::ontology::AmsAgentDescription;

    use crate::adt::{Adt, AgentReference, LocalAgentReference};

    use super::{ContainerView, PrivilegedAgent};

    impl PrivilegedAgent for AmsAgent {
        fn update_privileged(
            &mut self,
            container: &mut ContainerView<'_, '_>,
            environment: &mut Environment,
        ) {
            // Should never stop running.
            let _ = self.update(environment);

            use ember_fipa::ontology::ActionKind::*;
            while let Some(action) = self.actions.pop_front() {
                match action {
                    Register(r) => register_agent(r.ams, r.agent, &mut container.ladt),
                }
            }
        }
    }

    fn register_agent(_ams: AmsAgentDescription, agent: AmsAgentDescription, adt: &mut Adt) {
        // TODO: Check that the ams for which the action is meant is this one.

        use alloc::collections::btree_map::Entry;

        let aid: Aid = match agent.name.map(|n| n.parse()) {
            Some(Ok(aid)) => aid,
            Some(Err(e)) => {
                log::error!("Cannot register agent: {e}");
                return;
            }
            None => {
                log::error!("Cannot register an agent without a name.");
                return;
            }
        };
        if !aid.is_local() {
            log::error!("Cannot register agent that is not local to the ams.");
        }
        let name = aid.local_name().to_string();
        log::trace!("Trying to registering agent `{name}`.");
        match adt.entry(name.clone()) {
            Entry::Vacant(entry) => {
                entry.insert(AgentReference::Local(LocalAgentReference {
                    inbox: Vec::new(),
                }));
                log::info!("Agent `{}` successfully registered.", &name);
            }
            Entry::Occupied(_) => {
                log::error!("Cannot register agent `{aid}` as it is already registered.");
            }
        }
    }
}
