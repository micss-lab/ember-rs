use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::collections::VecDeque;

use crate::agent::{Agent, AmsAgent};
use crate::context::ContainerContext;
use crate::util::sync::AtomicBool;

static MAIN_CONTAINER_CREATED: AtomicBool = AtomicBool::new(false);

pub struct Container<K = Main> {
    /// Ams agent required for for a main container.
    ams: AmsAgent,
    /// Agents managed by this container.
    agents: VecDeque<Box<dyn AgentLike>>,
    /// Sub-containers managed by this container.
    _containers: K,
}
/// The container is a main container and can contain sub-containers.
#[derive(Default)]
pub struct Main(VecDeque<Container<Sub>>);
/// The container is a sub-container, thus cannot contain more sub-containers.
pub struct Sub;

pub trait AgentLike: 'static {
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

impl Default for Container {
    fn default() -> Self {
        check_and_set_created();
        Self {
            ams: AmsAgent::new(),
            agents: VecDeque::default(),
            _containers: Main::default(),
        }
    }
}

fn check_and_set_created() {
    if MAIN_CONTAINER_CREATED.compare_and_swap(false, true) {
        panic!("Can only create a single instance of the main container.");
    }
}
