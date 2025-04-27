use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::vec::Vec;

use crate::acl::message::Message;
use crate::adt::Adt;
use crate::agent::{Agent, Aid, AmsAgent};
use crate::context::ContainerContext;

mod mts;

pub struct Container {
    /// Agents managed by this container.
    agents: VecDeque<Box<dyn AgentLike>>,
    /// Ams agent managing this cotainers.
    ams: AmsAgent,
    /// Register of agents running on this platform.
    ladt: Adt,
}

pub trait AgentLike: 'static {
    fn update(&mut self, context: &mut ContainerContext) -> bool;

    fn get_name(&self) -> Cow<str>;

    fn get_aid(&self) -> Aid;
}

impl Container {
    pub fn start(mut self) -> Result<(), Box<dyn core::error::Error>> {
        loop {
            let should_stop = self.poll()?;
            if should_stop {
                break Ok(());
            }
        }
    }

    fn poll_associated_agents(&mut self) -> Result<(), Box<dyn core::error::Error>> {
        let mut context =
            ContainerContext::new(self.messages_for_agent(&Aid::ams()).unwrap_or_default());
        self.ams.update(&mut context);
        self.ams.perform_platform_actions(&mut self.ladt);
        Ok(())
    }

    pub fn poll(&mut self) -> Result<bool, Box<dyn core::error::Error>> {
        // Iterate over all agents once, only rescheduling agents that are not removed.
        let mut amount = self.agents.len();

        while let Some(mut agent) = self.agents.pop_front() {
            self.poll_associated_agents()?;

            let mut context = ContainerContext::new(
                self.messages_for_agent(&agent.get_aid())
                    .unwrap_or_default(),
            );

            let finished = agent.update(&mut context);

            // Handle all messages the agent wants to send.
            for message in context.message_outbox.into_iter() {
                mts::send_message(message, &mut self.ladt);
            }

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

    fn messages_for_agent(&mut self, aid: &Aid) -> Option<Vec<Message>> {
        use crate::acl::message::MessageKind;
        Some(
            core::mem::take(&mut self.ladt.get_mut(aid)?.inbox)
                .into_iter()
                .map(|m| match m.message {
                    MessageKind::Structured(m) => m,
                })
                .collect(),
        )
    }

    pub fn with_agent<E: 'static>(mut self, agent: Agent<E>) -> Self {
        self.add_agent(agent);
        self
    }

    pub fn add_agent<E: 'static>(&mut self, agent: Agent<E>) {
        self.agents.push_back(Box::new(agent));
    }
}

impl Default for Container {
    fn default() -> Self {
        let ams = AmsAgent::new();
        let ladt = Adt::new(&ams);
        Self {
            agents: VecDeque::default(),
            ams,
            ladt,
        }
    }
}
