use alloc::borrow::Cow;
use alloc::collections::vec_deque::VecDeque;

use lazy_static::lazy_static;

use ember_core::agent::Agent as AgentTrait;
use ember_core::environment::Environment;
use ember_core::message::Performative;
use ember_core::message::filter::MessageFilter;

use crate::ontology::{ActionKind, AgentManagementOntology};

#[derive(Default)]
pub struct AmsAgent {
    /// Display a message on agent startup.
    started: bool,

    /// Queue of actions to be performed using the hosting platform.
    pub actions: VecDeque<ActionKind>,
}

impl AgentTrait for AmsAgent {
    fn update(&mut self, environment: &mut Environment) -> bool {
        lazy_static! {
            static ref MESSAGE_FILTER: MessageFilter = MessageFilter::and([
                MessageFilter::performative(Performative::Request),
                MessageFilter::ontology(AgentManagementOntology::name()),
            ]);
        }

        if !self.started {
            log::debug!("Ams agent has registered.");
            self.started = true;
        }

        while let Some(message) = environment.receive_message(Some(Cow::Borrowed(&MESSAGE_FILTER)))
        {
            let action_kind = match AgentManagementOntology::decode_message(message) {
                Ok(k) => k,
                Err(e) => {
                    log::error!("Could not decode message: {e:?}");
                    todo!()
                }
            };
            self.actions.push_back(action_kind);
        }

        false
    }

    fn get_name(&self) -> Cow<str> {
        Cow::Borrowed("ams")
    }
}
