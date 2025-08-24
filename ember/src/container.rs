use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::string::ToString;
use alloc::vec::Vec;

#[cfg(feature = "acc-espnow")]
use esp_wifi::esp_now;

use ember_core::context::{ContainerContext, MessageStore};
use ember_core::message::MessageEnvelope;

use ember_core::agent::Agent;
use ember_core::agent::aid::Aid;

use crate::adt::{Adt, AgentReference};
use crate::agent::AmsAgent;

use self::mts::Mts;

mod mts;

pub struct Container<'a, 'c> {
    /// Agents managed by this container.
    agents: VecDeque<Box<dyn Agent + 'a>>,
    /// Ams agent managing this cotainers.
    ams: AmsAgent,
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

    fn poll_associated_agents(&mut self) -> Result<(), Box<dyn core::error::Error>> {
        let mut context = ContainerContext::new(
            self.messages_for_agent(Aid::ams().local_name())
                .unwrap_or_default(),
        );
        self.ams.update(&mut context);
        self.ams.perform_platform_actions(&mut self.ladt);
        Ok(())
    }

    pub fn poll(&mut self) -> Result<bool, Box<dyn core::error::Error>> {
        // Iterate over all agents once, only rescheduling agents that are not removed.
        let mut amount = self.agents.len();

        // Poll the message transport system.
        self.mts.receive_messages(&mut self.ladt);

        while let Some(mut agent) = self.agents.pop_front() {
            self.poll_associated_agents()?;

            let mut context = ContainerContext::new(
                self.messages_for_agent(agent.get_name())
                    .unwrap_or_default(),
            );

            let finished = agent.update(&mut context);

            // Handle all messages the agent wants to send.
            for message in context.message_outbox.into_iter() {
                self.mts.send_message(message, &mut self.ladt);
            }

            self.return_unhandled_messages(agent.get_name(), context.message_inbox);

            if context.should_stop {
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
        Some(
            core::mem::take(&mut self.ladt.get_local_mut(agent_name.as_ref())?.inbox)
                .into_iter()
                .collect(),
        )
    }

    fn return_unhandled_messages(&mut self, agent_name: impl AsRef<str>, messages: MessageStore) {
        if messages.is_empty() {
            return;
        }
        self.ladt
            .get_local_mut(agent_name.as_ref())
            .expect("agent returning messages should be in ladt")
            .inbox
            .extend(messages)
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

impl Container<'_, '_> {
    #[cfg(feature = "acc-http")]
    pub fn with_http(mut self, port: u16) -> Self {
        self.mts.enable_http(port);
        self
    }
}

impl<'c> Container<'_, 'c> {
    #[cfg(feature = "acc-espnow")]
    pub fn with_espnow(
        mut self,
        sender: Option<esp_now::EspNowSender<'c>>,
        receiver: Option<esp_now::EspNowReceiver<'c>>,
    ) -> Self {
        self.mts.enable_espnow(sender, receiver);
        self
    }
}

impl Default for Container<'_, '_> {
    fn default() -> Self {
        let ams = AmsAgent::new();
        let ladt = Adt::new(&ams);
        Self {
            agents: VecDeque::default(),
            ams,
            ladt,
            mts: Mts::new(),
        }
    }
}
