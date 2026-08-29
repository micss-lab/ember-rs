use alloc::borrow::Cow;
use alloc::vec::Vec;

use ember_core::agent::Agent;
use ember_core::environment::{Environment, SendCallbacks};
use ember_core::message::TransportMessage;
use ember_fipa::agent::ams::AmsAgent;

use crate::adt::Adt;

use super::mts::Mts;

/// Privileged agents able to modify the container/platform directly.
pub(super) trait PrivilegedAgent: Agent {
    fn update_privileged(
        &mut self,
        container: &mut ContainerView<'_>,
        environment: &mut Environment,
    );

    fn should_update(&self, _container: &ContainerView<'_>) -> bool {
        true
    }
}

pub(super) struct ContainerView<'a> {
    pub(super) ladt: &'a mut Adt,
    pub(super) pending_sends: &'a mut Vec<(TransportMessage, SendCallbacks)>,
}

#[derive(Default)]
pub(super) struct PrivilegedAgents<'c> {
    ams: AmsAgent,
    pub(super) mts: Mts<'c>,
}

impl<'c> PrivilegedAgents<'c> {
    pub(super) fn agent_names(&self) -> impl IntoIterator<Item = Cow<'_, str>> + '_ {
        [self.ams.get_name(), self.mts.get_name()]
    }

    pub(super) fn poll(&mut self, container: &mut ContainerView<'_>) {
        fn poll_agent(agent: &mut impl PrivilegedAgent, container: &mut ContainerView<'_>) {
            if !agent.should_update(container) {
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
            for entry in environment.message_outbox.into_iter() {
                container.pending_sends.push(entry);
            }

            container
                .ladt
                .return_unhandled_messages(agent.get_name(), environment.message_inbox);
        }

        poll_agent(&mut self.ams, &mut *container);
        poll_agent(&mut self.mts, &mut *container);
    }
}

mod mts {
    use ember_core::environment::Environment;

    use crate::container::mts::Mts;

    use super::{ContainerView, PrivilegedAgent};

    impl PrivilegedAgent for Mts<'_> {
        fn update_privileged(
            &mut self,
            container: &mut ContainerView<'_>,
            environment: &mut Environment,
        ) {
            for (message, callbacks) in core::mem::take(container.pending_sends) {
                self.route(message, callbacks, container.ladt);
            }

            #[cfg(feature = "acc")]
            {
                use ember_acc::{Acc, SendCallbacks};
                while let Some(mut message) = self.channels.receive(environment) {
                    let envelope = &mut message.envelopes.base;
                    // TODO: Do this according to the fipa spec by pushing a new envelope.
                    // Set the to parameter to the local address of the agent.
                    envelope.to = core::mem::take(&mut envelope.to)
                        .into_iter()
                        .map(|t| t.to_local())
                        .collect();

                    // Deliver the message as if it was to the local agent. Not a
                    // fresh outbound send, nothing to attach callbacks to.
                    self.route(message, SendCallbacks::default(), container.ladt);
                }
            }
            let _ = environment;
        }
    }
}

mod ams {
    use alloc::string::ToString;
    use alloc::vec::Vec;

    use ember_core::agent::Agent;
    use ember_core::agent::aid::Aid;
    use ember_core::environment::Environment;
    use ember_fipa::agent::ams::AmsAgent;
    use ember_fipa::ontology::AmsAgentDescription;

    use crate::adt::{Adt, AgentReference, LocalAgentReference};

    use super::{ContainerView, PrivilegedAgent};

    impl PrivilegedAgent for AmsAgent {
        fn update_privileged(
            &mut self,
            container: &mut ContainerView<'_>,
            environment: &mut Environment,
        ) {
            // Should never stop running.
            let _ = self.update(environment);

            use ember_fipa::ontology::ActionKind::*;
            while let Some(action) = self.actions.pop_front() {
                match action {
                    Register(r) => register_agent(r.ams, r.agent, container.ladt),
                }
            }
        }

        fn should_update(&self, container: &ContainerView<'_>) -> bool {
            container.ladt.agent_has_message(self.get_name())
        }
    }

    fn register_agent(_ams: AmsAgentDescription, agent: AmsAgentDescription, adt: &mut Adt) {
        // TODO: Check that the ams for which the action is meant is this one.

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
        if adt.contains_key(&name) {
            log::error!("Cannot register agent `{aid}` as it is already registered.");
        } else {
            adt.insert(
                name.clone(),
                AgentReference::Local(LocalAgentReference { inbox: Vec::new() }),
            );
            log::info!("Agent `{}` successfully registered.", name);
        }
    }
}
