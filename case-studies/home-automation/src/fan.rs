use no_std_framework_core::{
    behaviour::{Context, TickerBehaviour},
    Agent,
};

use self::ontology::FanAction;
use crate::util::wrap_message;

pub use self::ontology::FanState;

pub fn fan_agent() -> Agent<FanState, ()> {
    Agent::new("fan", FanState::default()).with_behaviour(FanInteractions)
}

pub mod ontology {
    use alloc::{string::String, vec::Vec};

    use no_std_framework_core::{
        acl::{
            codec::{AgentActionCodec, ConceptCodec, ConstantCodec, DecodeError},
            message::{Content, Message, Performative, Receiver},
            sl::{AgentAction, Concept, ConceptParameters},
        },
        Aid,
    };
    use serde::{Deserialize, Serialize};

    pub struct FanOntology;

    pub struct FanMessage {
        pub action: FanAction,
    }

    pub enum FanAction {
        Toggle,
    }

    #[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
    pub enum FanState {
        On,
        #[default]
        Off,
    }

    impl FanOntology {
        pub fn name() -> &'static str {
            "Fan-Ontology"
        }

        pub fn decode_sl_message<T: AgentActionCodec>(message: Message) -> T {
            if message.ontology.is_none_or(|o| o != Self::name()) {
                panic!("message has incorrect ontology");
            }
            let Content::Sl(content) = message.content else {
                panic!("received incorrect content type");
            };

            AgentActionCodec::from_content(content).expect("failed to parse content")
        }

        pub fn decode_message<T: for<'d> Deserialize<'d>>(message: Message) -> T {
            let Content::Bytes(content) = message.content else {
                panic!("received incorrect content type");
            };

            postcard::from_bytes(&content).expect("failed to parse content")
        }
    }

    impl AgentActionCodec for FanMessage {
        fn from_agent_action(
            agent_action: AgentAction,
        ) -> Result<Self, no_std_framework_core::acl::codec::DecodeError> {
            let action: FanAction = ConceptCodec::from_term(agent_action.action)?;

            Ok(FanMessage { action })
        }

        fn into_agent_action(self) -> AgentAction {
            AgentAction {
                agent: String::from("fan").into_term(),
                action: self.action.into_term(),
            }
        }
    }

    impl ConceptCodec for FanAction {
        fn from_concept(
            concept: Concept,
        ) -> Result<Self, no_std_framework_core::acl::codec::DecodeError> {
            if !concept.parameters.is_empty() {
                return Err(DecodeError::InvalidLength(concept.parameters.len()));
            }

            Ok(match concept.symbol.as_slice() {
                b"toggle" => FanAction::Toggle,
                _ => return Err(DecodeError::UnknownConcept(concept.symbol)),
            })
        }

        fn into_concept(self) -> Concept {
            Concept {
                symbol: match self {
                    Self::Toggle => "toggle".into(),
                },
                parameters: ConceptParameters::Positional(Vec::with_capacity(0)),
            }
        }
    }

    impl FanMessage {
        pub fn into_message(self) -> Message {
            Message {
                performative: Performative::Request,
                sender: None,
                receiver: Receiver::Single(Aid::local("fan")),
                reply_to: None,
                ontology: Some(FanOntology::name().into()),
                content: Content::Sl(self.into_content()),
            }
        }
    }

    impl FanState {
        pub fn into_message(self) -> Message {
            Message {
                performative: Performative::Inform,
                sender: None,
                receiver: Receiver::Single(Aid::local("control")),
                reply_to: None,
                ontology: Some(FanOntology::name().into()),
                content: Content::Bytes(postcard::to_allocvec(&self).unwrap()),
            }
        }
    }
}

impl FanState {
    fn toggle(&mut self) {
        *self = match self {
            Self::On => Self::Off,
            Self::Off => Self::On,
        }
    }
}

struct FanInteractions;

impl TickerBehaviour for FanInteractions {
    type AgentState = FanState;

    type Event = ();

    fn interval(&self) -> core::time::Duration {
        core::time::Duration::from_millis(250)
    }

    fn action(&mut self, ctx: &mut Context<Self::Event>, state: &mut Self::AgentState) {
        use ontology::FanMessage;

        let Some(message) = ctx.receive_message(None) else {
            ctx.block_behaviour();
            return;
        };
        let message: FanMessage = ontology::FanOntology::decode_sl_message(message);

        match message.action {
            FanAction::Toggle => {
                log::info!("Toggling fan.");
                state.toggle();
                ctx.send_message(wrap_message(state.into_message()));
            }
        }
    }

    fn is_finished(&self) -> bool {
        false
    }
}
