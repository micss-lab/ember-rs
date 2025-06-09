use alloc::borrow::Cow;
use alloc::format;
use alloc::string::{String, ToString};

use crate::acl::message::{Message, MessageEnvelope, Performative};
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
            behaviours: ParallelBehaviourQueue::new_empty(FinishStrategy::Never),
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
        self.behaviours.add_behaviour(behaviour);
        id
    }
}

impl<E: 'static> AgentLike for Agent<E> {
    fn update(&mut self, ctx: &mut ContainerContext) -> bool {
        use crate::acl::codec::AgentActionCodec;
        use crate::behaviour::complex::scheduler::BehaviourScheduler;
        use AgentState::*;

        // log::trace!("Ticking agent `{}`", self.name);

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
                                name: Some(format!("{}@local", self.get_name())),
                            },
                        }
                        .into_content()
                        .into(),
                    },
                ));
                log::debug!("Sending ams register request for agent `{}`.", self.name);
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Aid {
    name: (String, AgentPlatform),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AgentPlatform {
    Local,
    Public(String),
}

pub type TransportAddress = String;

impl Aid {
    pub fn local(agent: impl ToString) -> Self {
        Self {
            name: (agent.to_string(), AgentPlatform::Local),
        }
    }

    pub fn ams() -> Self {
        Self::local("ams")
    }

    pub fn general(agent: impl ToString, platform: impl ToString) -> Self {
        let platform = {
            let platform = platform.to_string();
            match platform.as_str() {
                "local" => AgentPlatform::Local,
                _ => AgentPlatform::Public(platform),
            }
        };

        Self {
            name: (agent.to_string(), platform),
        }
    }

    pub fn is_local(&self) -> bool {
        matches!(self.name.1, AgentPlatform::Local)
    }

    pub fn to_local(self) -> Self {
        Self::local(self.name.0)
    }

    pub(crate) fn to_transport_address(&self) -> TransportAddress {
        format!("http://{}/acc", self.name.1)
    }
}

impl core::str::FromStr for Aid {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((name, ap)) = s.split_once('@') else {
            return Err(
                "Failed to parse aid: incorrect format (expected <agent-name>@<agent-platform>)"
                    .into(),
            );
        };
        Ok(Self {
            name: (name.to_string(), ap.parse()?),
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
        write!(f, "{}@{}", self.name.0, self.name.1)
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

impl serde::Serialize for Aid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut aid = serializer.serialize_struct("agent-identifier", 1)?;
        aid.serialize_field("name", &self.to_string())?;
        aid.end()
    }
}

impl<'de> serde::Deserialize<'de> for Aid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        enum Field {
            Name,
        }

        impl<'de> serde::Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> serde::de::Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                        formatter.write_str("`name`")
                    }

                    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        Ok(match v {
                            "name" => Field::Name,
                            _ => return Err(serde::de::Error::unknown_field(v, FIELDS)),
                        })
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct AidVisitor;

        impl<'de> serde::de::Visitor<'de> for AidVisitor {
            type Value = Aid;

            fn expecting(&self, formatter: &mut alloc::fmt::Formatter) -> alloc::fmt::Result {
                formatter.write_str("struct agent-identifier")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut name = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Name => {
                            if name.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name = Some(map.next_value()?);
                        }
                    }
                }

                let name: String = name.ok_or_else(|| serde::de::Error::missing_field("name"))?;

                Ok(name.parse().map_err(serde::de::Error::custom)?)
            }
        }

        const FIELDS: &[&str] = &["name"];
        deserializer.deserialize_struct("agent-identifier", FIELDS, AidVisitor)
    }
}
