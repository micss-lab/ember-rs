use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::agent::Agent;
use crate::behaviour::Context;

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
        'start: loop {
            for agent in self.agents.iter_mut() {
                let mut context = Context::default();

                log::trace!("Agent `{}` update:", agent.name);
                agent.update(&mut context);

                let Context {
                    container: context, ..
                } = context;

                if context.should_stop {
                    break 'start Ok(());
                }
            }
        }
    }
}
