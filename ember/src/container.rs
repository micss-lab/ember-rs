use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::string::ToString;
use alloc::vec::Vec;

#[cfg(feature = "acc-espnow")]
use esp_wifi::esp_now;

use ember_core::agent::Agent;
use ember_core::agent::aid::Aid;
use ember_core::environment::{Environment, MessageStore};
use ember_core::message::MessageEnvelope;

use crate::adt::{Adt, AgentReference};

use self::mts::Mts;
use self::privileged::{ContainerView, PrivilegedAgents};

mod mts;
mod privileged;

pub struct Container<'a, 'c> {
    /// Agents managed by this container.
    agents: VecDeque<Box<dyn Agent + 'a>>,
    /// Store of privileged agents able to modify the container directly.
    privileged: PrivilegedAgents,
    /// Register of agents running on this platform.
    ladt: Adt,
    /// Message transport service.
    mts: Mts<'c>,
}

impl Container<'_, '_> {
    pub fn start(mut self) -> Result<(), Box<dyn core::error::Error>> {
        loop {
            let should_stop = self.poll()?;
            if should_stop {
                break Ok(());
            }
        }
    }

    pub fn poll(&mut self) -> Result<bool, Box<dyn core::error::Error>> {
        // Iterate over all agents once, only rescheduling agents that are not removed.
        let mut amount = self.agents.len();

        // Poll the message transport system.
        self.mts.receive_messages(&mut self.ladt);

        // Poll privileged agents associated to this container.
        self.privileged.poll(&mut ContainerView {
            ladt: &mut self.ladt,
            mts: &mut self.mts,
        });

        while let Some(mut agent) = self.agents.pop_front() {
            let mut context = Environment::new(
                self.messages_for_agent(agent.get_name())
                    .unwrap_or_default(),
            );

            let finished = agent.update(&mut context);

            // Handle all messages the agent wants to send.
            for message in context.message_outbox.into_iter() {
                self.mts.send_message(message, &mut self.ladt);
            }

            self.return_unhandled_messages(agent.get_name(), context.message_inbox);

            if context.stop_platform {
                return Ok(true);
            }

            if !finished {
                self.agents.push_back(agent);
            }

            amount -= 1;
            if amount == 0 {
                break;
            }
        }

        Ok(false)
    }

    fn messages_for_agent(&mut self, agent_name: impl AsRef<str>) -> Option<Vec<MessageEnvelope>> {
        self.ladt.messages_for_agent(agent_name)
    }

    fn return_unhandled_messages(&mut self, agent_name: impl AsRef<str>, messages: MessageStore) {
        self.ladt.return_unhandled_messages(agent_name, messages);
    }

    pub fn with_agent_proxy(mut self, local_name: impl ToString, agent_proxy: Aid) -> Self {
        self.add_agent_proxy(local_name, agent_proxy);
        self
    }

    pub fn add_agent_proxy(&mut self, local_name: impl ToString, agent_proxy: Aid) {
        let local_name = local_name.to_string();
        if self
            .ladt
            .insert(local_name.clone(), AgentReference::Proxy(agent_proxy))
            .is_some()
        {
            log::error!("Agent(-proxy) with name {local_name} exists.")
        }
    }
}

impl<'a> Container<'a, '_> {
    pub fn with_agent<'b>(mut self, agent: impl Agent + 'b) -> Self
    where
        'b: 'a,
    {
        self.add_agent(agent);
        self
    }

    pub fn add_agent<'b>(&mut self, agent: impl Agent + 'b)
    where
        'b: 'a,
    {
        self.agents.push_back(Box::new(agent));
    }
}

impl<'c> Container<'_, 'c> {
    #[cfg(feature = "acc-http")]
    pub fn enable_http(&mut self, port: u16) {
        self.mts.enable_http(port);
    }

    #[cfg(feature = "acc-http")]
    pub fn with_http(mut self, port: u16) -> Self {
        self.enable_http(port);
        self
    }

    #[cfg(feature = "acc-espnow")]
    pub fn enable_espnow(
        &mut self,
        sender: Option<esp_now::EspNowSender<'c>>,
        receiver: Option<esp_now::EspNowReceiver<'c>>,
    ) {
        self.mts.enable_espnow(sender, receiver);
    }

    #[cfg(feature = "acc-espnow")]
    pub fn with_espnow(
        mut self,
        sender: Option<esp_now::EspNowSender<'c>>,
        receiver: Option<esp_now::EspNowReceiver<'c>>,
    ) -> Self {
        self.enable_espnow(sender, receiver);
        self
    }

    #[cfg(feature = "acc-custom")]
    pub fn enable_custom_acc(&mut self, custom: Box<dyn ember_acc::Acc + 'c>) {
        self.mts.enable_custom_acc(custom);
    }

    #[cfg(feature = "acc-custom")]
    pub fn with_custom_acc(mut self, custom: Box<dyn ember_acc::Acc + 'c>) -> Self {
        self.enable_custom_acc(custom);
        self
    }
}

impl Default for Container<'_, '_> {
    fn default() -> Self {
        let privileged = PrivilegedAgents::default();
        let ladt = Adt::new(privileged.agent_names());
        Self {
            agents: VecDeque::default(),
            privileged,
            ladt,
            mts: Mts::new(),
        }
    }
}
