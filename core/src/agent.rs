use alloc::borrow::Cow;
use alloc::format;
use alloc::string::{String, ToString};

use crate::acl::message::{Message, MessageEnvelope, Performative};
use crate::behaviour::complex::queue::{BehaviourScheduler, ScheduleStrategy};
use crate::behaviour::parallel::{FinishStrategy, ParallelBehaviourQueue};
use crate::behaviour::{BehaviourId, IntoBehaviour};
use crate::container::AgentLike;
use crate::context::{ContainerContext, Context};
use crate::fipa::{self, AmsAgentDescription, RegisterAction};

pub(crate) use self::ams::AmsAgent;

mod ams;

enum AgentState {
    Initiated,
    Active,
    Suspended,
    // TODO: Implement these.
    // Waiting,
    // Transit,
}

pub struct Agent<E> {
    pub(crate) name: String,
    behaviours: ParallelBehaviourQueue<E>,
    state: AgentState,
}

impl<E: 'static> Agent<E> {
    pub fn new(name: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            behaviours: ParallelBehaviourQueue::new(FinishStrategy::Never),
            state: AgentState::Initiated,
        }
    }
}

impl<E: 'static> Agent<E> {
    pub fn with_behaviour<K>(mut self, behaviour: impl IntoBehaviour<K, Event = E>) -> Self {
        self.add_behaviour(behaviour);
        self
    }

    pub fn add_behaviour<K>(&mut self, behaviour: impl IntoBehaviour<K, Event = E>) -> BehaviourId {
        let behaviour = behaviour.into_behaviour();
        let id = behaviour.id();
        self.behaviours.schedule(behaviour, ScheduleStrategy::End);
        id
    }
}

impl<E: 'static> AgentLike for Agent<E> {
    fn update(&mut self, ctx: &mut ContainerContext) -> bool {
        use crate::acl::codec::AgentActionCodec;
        use AgentState::*;

        log::trace!("Ticking agent `{}`", self.name);

        match self.state {
            Initiated => {
                // First register the agent with the ams.
                let ams_aid = Aid::local("ams");

                ctx.send_message(MessageEnvelope::new(
                    ams_aid.clone(),
                    Message {
                        performative: Performative::Request,
                        sender: None,
                        receiver: ams_aid.into(),
                        reply_to: None,
                        ontology: Some(fipa::ManagementOntology::name().to_string()),
                        content: RegisterAction {
                            ams: AmsAgentDescription { name: None },
                            agent: AmsAgentDescription {
                                name: Some(format!("{}@local", self.get_name().to_string())),
                            },
                        }
                        .into_content()
                        .into(),
                    },
                ));
                log::debug!("Sending ams register request.");
                self.state = Active;
                return false;
            }
            Active => (),
            Suspended => return false,
        }

        let mut context = Context::new_using_container(&mut *ctx);
        self.behaviours.action(&mut context);

        ctx.merge(context.container);

        context.agent.should_remove
    }

    fn get_name(&self) -> Cow<str> {
        Cow::from(&self.name)
    }

    fn get_aid(&self) -> Aid {
        Aid::local(self.get_name())
    }
}

#[derive(Debug, Clone, PartialOrd, Ord)]
pub enum Aid {
    Ams,
    Other { name: String, ap: AgentPlatform },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AgentPlatform {
    Local,
    Public(String),
}

impl Aid {
    pub(crate) fn local(agent: impl ToString) -> Self {
        let agent = agent.to_string();
        if agent == "ams" {
            return Self::ams();
        }
        Self::Other {
            name: agent.to_string(),
            ap: AgentPlatform::Local,
        }
    }

    pub(crate) fn ams() -> Self {
        Self::Ams
    }
}

impl core::str::FromStr for Aid {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "ams@local" {
            return Ok(Self::Ams);
        }
        let Some((name, ap)) = s.split_once('@') else {
            return Err(
                "Failed to parse aid: incorrect format (expected <agent-name>@<agent-platform>)"
                    .into(),
            );
        };
        Ok(Self::Other {
            name: name.to_string(),
            ap: ap.parse()?,
        })
    }
}

impl core::str::FromStr for AgentPlatform {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "local" => Self::Local,
            s => Self::Public(s.to_string()),
        })
    }
}

impl core::fmt::Display for Aid {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Ams => write!(f, "ams@local"),
            Self::Other { name, ap } => write!(f, "{}@{}", name, ap),
        }
    }
}

impl core::fmt::Display for AgentPlatform {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Local => write!(f, "local"),
            Self::Public(v) => f.write_str(v),
        }
    }
}

impl PartialEq for Aid {
    fn eq(&self, other: &Self) -> bool {
        use Aid::*;
        match (self, other) {
            // AMS special case: both either Ams or Other { name: "ams", ap: Local }
            (Ams, Ams) => true,
            (Ams, Other { name, ap }) | (Other { name, ap }, Ams) => {
                name.eq_ignore_ascii_case("ams") && ap == &AgentPlatform::Local
            }
            // All other cases: structural equality
            (Other { name: n1, ap: ap1 }, Other { name: n2, ap: ap2 }) => n1 == n2 && ap1 == ap2,
        }
    }
}

impl Eq for Aid {}
