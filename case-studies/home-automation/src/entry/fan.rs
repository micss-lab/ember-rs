use no_std_framework_core::{
    behaviour::{Context, TickerBehaviour},
    Agent,
};

use self::ontology::FanAction;

pub fn fan_agent() -> Agent<FanState, ()> {
    Agent::new("fan", FanState::default()).with_behaviour(FanInteractions)
}

pub mod ontology {
    use alloc::{string::String, vec::Vec};

    use no_std_framework_core::acl::{
        codec::{AgentActionCodec, ConceptCodec, ConstantCodec, DecodeError},
        message::{Content, Message},
        sl::{AgentAction, Concept, ConceptParameters},
    };

    pub struct FanOntology;

    pub struct FanMessage {
        pub action: FanAction,
    }

    pub enum FanAction {
        Toggle,
    }

    impl FanOntology {
        pub fn name() -> &'static str {
            "Fan-Ontology"
        }

        pub fn decode_message(message: Message) -> Result<FanMessage, ()> {
            if !message.ontology.is_some_and(|o| o == Self::name()) {
                return Err(());
            }
            let Content::Sl(content) = message.content else {
                return Err(());
            };

            AgentActionCodec::from_content(content).map_err(|_| ())
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
}

#[derive(Default)]
pub enum FanState {
    #[default]
    On,
    Off,
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
        let Some(message) = ctx.receive_message(None) else {
            ctx.block_behaviour();
            return;
        };
        let message = ontology::FanOntology::decode_message(message).unwrap();

        match message.action {
            FanAction::Toggle => {
                log::info!("Toggling fan.");
                state.toggle()
            }
        }
    }

    fn is_finished(&self) -> bool {
        false
    }
}
