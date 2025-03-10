use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::collections::VecDeque;

use crate::agent::{Agent, AmsAgent};
use crate::context::ContainerContext;
use crate::util::sync::AtomicBool;

use self::kind::{ContainerKind, Main};

static MAIN_CONTAINER_CREATED: AtomicBool = AtomicBool::new(false);

pub struct Container<K = Main> {
    /// Agents managed by this container.
    agents: VecDeque<Box<dyn AgentLike>>,
    kind: K,
}

mod kind {
    use alloc::boxed::Box;
    use alloc::collections::VecDeque;

    use crate::agent::AmsAgent;
    use crate::context::ContainerContext;

    use super::{AgentLike, Container};

    /// The container is a main container and can contain sub-containers.
    pub struct Main {
        /// Ams agent required for for a main container.
        pub(super) ams: AmsAgent,
        /// Sub-containers managed by this main container.
        pub(super) containers: VecDeque<Container<Sub>>,
    }
    /// The container is a sub-container, thus cannot contain more sub-containers.
    pub(super) struct Sub;

    pub trait ContainerKind {
        fn poll_associated_agents(&mut self) -> Result<(), Box<dyn core::error::Error>> {
            Ok(())
        }

        fn poll_sub_containers(&mut self) -> Result<(), Box<dyn core::error::Error>> {
            Ok(())
        }
    }

    impl ContainerKind for Main {
        fn poll_associated_agents(&mut self) -> Result<(), Box<dyn core::error::Error>> {
            let mut context = ContainerContext::new();
            self.ams.update(&mut context);
            Ok(())
        }

        fn poll_sub_containers(&mut self) -> Result<(), Box<dyn core::error::Error>> {
            todo!()
        }
    }
    impl ContainerKind for Sub {}
}
pub trait AgentLike: 'static {
    fn update(&mut self, context: &mut ContainerContext) -> bool;

    #[allow(unused)]
    fn get_name(&self) -> Cow<str>;
}

impl<K> Container<K> {
    pub fn with_agent<M: 'static>(mut self, agent: Agent<M>) -> Self {
        self.add_agent(agent);
        self
    }

    pub fn add_agent<M: 'static>(&mut self, agent: Agent<M>) {
        self.agents.push_back(Box::new(agent));
    }
}

impl<K> Container<K>
where
    K: ContainerKind,
{
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

        self.kind.poll_associated_agents()?;

        while let Some(mut agent) = self.agents.pop_front() {
            let mut context = ContainerContext::new();

            let finished = agent.update(&mut context);

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

            // Give associated agents a priority, for example, for agent name
            // resolution, creation, deletion, etc.
            self.kind.poll_associated_agents()?;
        }
        self.kind.poll_sub_containers()?;

        Ok(false)
    }
}

impl Default for Container {
    fn default() -> Self {
        check_and_set_created();
        Self {
            agents: VecDeque::default(),
            kind: Main {
                ams: AmsAgent::new(),
                containers: VecDeque::default(),
            },
        }
    }
}

fn check_and_set_created() {
    if MAIN_CONTAINER_CREATED.compare_and_swap(false, true) {
        panic!("Can only create a single instance of the main container.");
    }
}
