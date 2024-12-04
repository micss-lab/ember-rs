use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::agent::Agent;

#[derive(Default)]
pub struct Container {
    agents: Vec<Agent>,
}

impl Container {
    pub fn with_agent(mut self, agent: Agent) -> Self {
        self.add_agent(agent);
        self
    }

    pub fn add_agent(&mut self, agent: Agent) {
        self.agents.push(agent);
    }

    pub fn start(mut self) -> Result<(), Box<dyn core::error::Error>> {
        loop {
            for agent in self.agents.iter_mut() {
                agent.update();
            }
        }
    }
}
