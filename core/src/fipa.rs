use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::acl::codec::{AgentActionCodec, ConceptCodec, ConstantCodec, DecodeError};
use crate::acl::message::{Content, Message};
use crate::acl::sl::{AgentAction, Concept, ConceptParameters, Term};

pub struct ManagementOntology;

impl ManagementOntology {
    pub fn name() -> &'static str {
        "FIPA-Agent-Management"
    }
}

pub enum ActionKind {
    Register(RegisterAction),
}

#[derive(Debug)]
pub enum DecodeAsOntologyError {
    UnexpectedOntology,
    UnexpectedLanguage,
    UnsupportedMessage,
    #[allow(dead_code)]
    Decode(DecodeError),
}

impl ManagementOntology {
    pub(crate) fn decode_message(message: Message) -> Result<ActionKind, DecodeAsOntologyError> {
        if !message.ontology.is_some_and(|o| o == Self::name()) {
            return Err(DecodeAsOntologyError::UnexpectedOntology);
        }
        let Content::Sl(content) = message.content else {
            return Err(DecodeAsOntologyError::UnexpectedLanguage);
        };
        let mut agent_action: AgentAction =
            AgentActionCodec::from_content(content).map_err(DecodeAsOntologyError::Decode)?;
        let action: Concept =
            ConceptCodec::from_term(agent_action.action).map_err(DecodeAsOntologyError::Decode)?;

        Ok(match action.symbol.as_slice() {
            b"register" => {
                agent_action.action = Term::Concept(action);
                ActionKind::Register(
                    AgentActionCodec::from_agent_action(agent_action)
                        .map_err(DecodeAsOntologyError::Decode)?,
                )
            }
            _ => return Err(DecodeAsOntologyError::UnsupportedMessage),
        })
    }
}

pub(crate) struct RegisterAction {
    /// The ams to register with.
    pub(crate) ams: AmsAgentDescription,
    /// The agent being registered.
    pub(crate) agent: AmsAgentDescription,
}

struct Register {
    agent: AmsAgentDescription,
}

impl ConceptCodec for Register {
    fn from_concept(concept: Concept) -> Result<Self, DecodeError> {
        use ConceptParameters::*;
        if concept.symbol != "register" {
            return Err(DecodeError::UnknownConcept(concept.symbol));
        }
        let agent = match concept.parameters {
            Positional(mut items) => {
                if items.len() != 1 {
                    return Err(DecodeError::InvalidLength(items.len()));
                }
                ConceptCodec::from_term(items.remove(0))?
            }
            ByName(params) => {
                let mut result = None;
                for (key, value) in params {
                    match key.as_slice() {
                        b"agent" => result = Some(value),
                        _ => return Err(DecodeError::UnknownField(key)),
                    }
                }
                match result {
                    Some(agent) => ConceptCodec::from_term(agent)?,
                    None => return Err(DecodeError::MissingField("agent")),
                }
            }
        };
        Ok(Register { agent })
    }

    fn into_concept(self) -> Concept {
        Concept {
            symbol: "register".into(),
            parameters: ConceptParameters::ByName(vec![("agent".into(), self.agent.into_term())]),
        }
    }
}

impl AgentActionCodec for RegisterAction {
    fn from_agent_action(action: AgentAction) -> Result<Self, DecodeError> {
        let ams = ConceptCodec::from_term(action.agent)?;
        let agent = {
            let register: Register = ConceptCodec::from_term(action.action)?;
            register.agent
        };
        Ok(RegisterAction { ams, agent })
    }

    fn into_agent_action(self) -> AgentAction {
        AgentAction {
            agent: self.ams.into_term(),
            action: Register { agent: self.agent }.into_term(),
        }
    }
}

pub(crate) struct AmsAgentDescription {
    pub(crate) name: Option<String>,
}

impl ConceptCodec for AmsAgentDescription {
    fn from_concept(concept: Concept) -> Result<Self, DecodeError> {
        use ConceptParameters::*;
        if concept.symbol != "ams-agent-description" {
            return Err(DecodeError::UnknownConcept(concept.symbol));
        }
        let name = match concept.parameters {
            Positional(items) => {
                if items.len() > 1 {
                    return Err(DecodeError::InvalidLength(items.len()));
                }
                items
                    .into_iter()
                    .next()
                    .map(ConstantCodec::from_term)
                    .transpose()?
            }
            ByName(params) => {
                let mut result = None;
                for (key, value) in params {
                    match key.as_slice() {
                        b"name" => result = Some(value),
                        _ => return Err(DecodeError::UnknownField(key)),
                    }
                }
                result.map(ConstantCodec::from_term).transpose()?
            }
        };
        Ok(AmsAgentDescription { name })
    }

    fn into_concept(self) -> Concept {
        let mut parameters = Vec::new();
        if let Some(name) = self.name {
            parameters.push(("name".into(), name.into_term()))
        }
        Concept {
            symbol: "ams-agent-description".into(),
            parameters: ConceptParameters::ByName(parameters),
        }
    }
}
