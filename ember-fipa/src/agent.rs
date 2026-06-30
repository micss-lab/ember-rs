use alloc::borrow::Cow;
use alloc::format;

use alloc::string::ToString;
use ember_core::agent::aid::Aid;
use ember_core::environment::Environment;
use ember_core::message::content::fipa_sl::codec::AgentActionCodec;
use ember_core::message::{Message, Performative};

use crate::ontology::{AgentManagementOntology, AmsAgentDescription, RegisterAction};

pub mod ams;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExecutionState {
    #[default]
    Initiated,
    Active,
    // TODO: Implement these.
    // Suspended,
    // Waiting,
    // Transit,
}

#[derive(Default)]
pub struct FipaAgent {
    /// Whether the register message has been sent.
    ///
    /// TODO: Should the ams agent respond to a register message?
    registered: bool,
    state: ExecutionState,
}

impl FipaAgent {
    /// Ticks the fipa agent component returning the execution state of the agent.
    ///
    /// Because this behaviour is component based and not through inheritance, it is up to the
    /// super agent to adhere to this state.
    #[must_use]
    pub fn update(
        &mut self,
        environment: &mut Environment,
        agent_name: &Cow<'static, str>,
    ) -> ExecutionState {
        use ExecutionState::*;
        match self.state {
            Initiated if !self.registered => {
                // First register the agent with the ams.
                let ams_aid = Aid::local("ams");

                environment.send_message(
                    Message {
                        performative: Performative::Request,
                        sender: None,
                        receiver: ams_aid.into(),
                        reply_to: None,
                        ontology: Some(AgentManagementOntology::name().to_string()),
                        content: RegisterAction {
                            ams: AmsAgentDescription { name: None },
                            agent: AmsAgentDescription {
                                name: Some(format!("{agent_name}@local")),
                            },
                        }
                        .into_content()
                        .into(),
                    }
                    .wrap_with_envolope(),
                );
                log::debug!("Sending ams register request for agent `{agent_name}`.");
                self.registered = true;
            }
            Initiated => (),
            Active => (),
        }
        self.state
    }
}
