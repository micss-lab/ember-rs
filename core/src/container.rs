use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::agent::Agent;
use crate::behaviour::Context;
use crate::Actor;

#[derive(Default)]
pub struct Container {
    agents: Vec<Box<dyn Actor>>,
}

impl Container {
    pub fn with_agent(mut self, agent: impl Actor) -> Self {
        self.add_agent(agent);
        self
    }

    pub fn add_agent(&mut self, agent: impl Actor) {
        self.agents.push(Box::new(agent));
    }

    pub fn start(mut self) -> Result<(), Box<dyn core::error::Error>> {
        log::trace!("Starting the container.\r");
        'start: loop {
            log::trace!("Polling all agents.\r");
            for agent in self.agents.iter_mut() {
                let mut context = Context::new();

                log::trace!("Agent `{}` update:\r", agent.get_name());
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
