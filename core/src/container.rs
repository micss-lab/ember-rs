use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::format;

use crate::adt::Adt;
use crate::agent::{Agent, AmsAgent};
use crate::context::ContainerContext;
use crate::util::sync::AtomicBool;

use self::kind::{ContainerKind, Main};

mod mts;

static MAIN_CONTAINER_CREATED: AtomicBool = AtomicBool::new(false);

pub struct Container<K = Main> {
    /// Agents managed by this container.
    agents: VecDeque<Box<dyn AgentLike>>,
    kind: K,
}

mod kind {
    use alloc::collections::VecDeque;
    use alloc::format;
    use alloc::vec::Vec;
    use alloc::{borrow::Cow, boxed::Box};

    use crate::acl::message::Message;
    use crate::adt::Adt;
    use crate::agent::{Aid, AmsAgent};
    use crate::context::ContainerContext;

    use super::{AgentLike, Container};

    /// The container is a main container and can contain sub-containers.
    pub struct Main {
        /// Ams agent managing this and all sub-cotainers.
        pub(super) ams: AmsAgent,
        /// Sub-containers managed by this main container.
        pub(super) containers: VecDeque<Container<Sub>>,
        /// Register of agents running on this platform.
        pub(super) ladt: Adt,
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
            let mut context = ContainerContext::new(
                self.messages_for_agent(&Cow::Owned(format!("{}@local", self.ams.get_name())))
                    .unwrap_or_default(),
            );
            self.ams.update(&mut context);
            self.ams.perform_platform_actions(&mut self.ladt);
            Ok(())
        }

        fn poll_sub_containers(&mut self) -> Result<(), Box<dyn core::error::Error>> {
            // TODO: Implement sub-containers.
            Ok(())
        }
    }

    impl Main {
        pub(super) fn messages_for_agent(&mut self, aid: &Aid) -> Option<Vec<Message>> {
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
    }

    impl ContainerKind for Sub {}
}
pub trait AgentLike: 'static {
    fn update(&mut self, context: &mut ContainerContext) -> bool;

    #[allow(unused)]
    fn get_name(&self) -> Cow<str>;
}

impl Container<Main> {
    pub fn with_agent<E: 'static>(mut self, agent: Agent<E>) -> Self {
        self.add_agent(agent);
        self
    }

    pub fn add_agent<E: 'static>(&mut self, agent: Agent<E>) {
        self.agents.push_back(Box::new(agent));
    }
}

impl Container<Main> {
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
            self.kind.poll_associated_agents()?;

            let mut context = ContainerContext::new(
                self.kind
                    .messages_for_agent(&Cow::Owned(format!("{}@local", agent.get_name())))
                    .unwrap_or_default(),
            );

            let finished = agent.update(&mut context);

            // Handle all messages the agent wants to send.
            for message in context.message_outbox.into_iter() {
                mts::send_message(message, &mut self.kind.ladt);
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
        self.kind.poll_sub_containers()?;

        Ok(false)
    }
}

impl Default for Container {
    fn default() -> Self {
        check_and_set_created();
        let ams = AmsAgent::new();
        let ladt = Adt::new(&ams);
        Self {
            agents: VecDeque::default(),
            kind: Main {
                ams,
                containers: VecDeque::default(),
                ladt,
            },
        }
    }
}

fn check_and_set_created() {
    if MAIN_CONTAINER_CREATED.compare_and_swap(false, true) {
        panic!("Can only create a single instance of the main container.");
    }
}
