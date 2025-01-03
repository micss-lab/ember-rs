use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::collections::VecDeque;

use crate::context::ContainerContext;
use crate::Agent;

#[derive(Default)]
pub struct Container {
    agents: VecDeque<Box<dyn ContainerAgent>>,
}

pub trait ContainerAgent: 'static {
    fn update(&mut self, context: &mut ContainerContext) -> bool;

    #[allow(unused)]
    fn get_name(&self) -> Cow<str>;
}

impl Container {
    pub fn with_agent<M: 'static>(mut self, agent: Agent<M>) -> Self {
        self.add_agent(agent);
        self
    }

    pub fn add_agent<M: 'static>(&mut self, agent: Agent<M>) {
        self.agents.push_back(Box::new(agent));
    }

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

        while let Some(mut agent) = self.agents.pop_front() {
            let mut context = ContainerContext::new();

            let removed = agent.update(&mut context);

            if context.should_stop {
                return Ok(true);
            }

            if !removed {
                self.agents.push_back(agent);
            }

            amount -= 1;
            if amount == 0 {
                break;
            }
        }
        Ok(false)
    }
}
