use alloc::boxed::Box;

use no_std_framework_core::{
    behaviour::{
        parallel::{self, ParallelBehaviour},
        Behaviour, ComplexBehaviour, Context, IntoBehaviour, TickerBehaviour,
    },
    Agent,
};

use self::ontology::FanAction;

pub fn fan_agent() -> Agent<(), ()> {
    Agent::new("fan", ()).with_behaviour(Fan { state: false })
}

pub mod ontology {
    use alloc::vec::Vec;

    use no_std_framework_core::acl::{
        codec::{AgentActionCodec, ConceptCodec, ConstantCodec, DecodeError},
        message::{Content, Message},
        sl::{AgentAction, Concept, ConceptParameters},
    };

    pub struct FanOntology;

    pub struct FanMessage {
        pub fan: u32,
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
            let fan = {
                let fan: i32 = ConstantCodec::from_term(agent_action.agent)?;
                u32::try_from(fan).unwrap()
            };
            let action: FanAction = ConceptCodec::from_term(agent_action.action)?;

            Ok(FanMessage { fan, action })
        }

        fn into_agent_action(self) -> AgentAction {
            AgentAction {
                agent: i32::try_from(self.fan).unwrap().into_term(),
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

struct FanInteractions;

impl TickerBehaviour for FanInteractions {
    type AgentState = ();

    type Event = FanAction;

    fn interval(&self) -> core::time::Duration {
        core::time::Duration::from_millis(250)
    }

    fn action(&mut self, ctx: &mut Context<Self::Event>, _: &mut Self::AgentState) {
        let Some(message) = ctx.receive_message(None) else {
            ctx.block_behaviour();
            return;
        };
        let message = ontology::FanOntology::decode_message(message).unwrap();

        ctx.emit_event(message.action);
    }

    fn is_finished(&self) -> bool {
        false
    }
}

struct Fan {
    state: bool,
}

impl ComplexBehaviour for Fan {
    type AgentState = ();

    type Event = ();

    type ChildEvent = FanAction;

    fn handle_child_event(&mut self, event: Self::ChildEvent) {
        match event {
            FanAction::Toggle => {
                log::info!("Toggling fan.");
                self.state = !self.state
            }
        }
    }
}

impl ParallelBehaviour for Fan {
    fn finish_strategy(&self) -> parallel::FinishStrategy {
        parallel::FinishStrategy::Never
    }

    fn initial_behaviours(
        &self,
    ) -> impl IntoIterator<
        Item = Box<dyn Behaviour<AgentState = Self::AgentState, Event = Self::ChildEvent>>,
    > {
        [FanInteractions.into_behaviour()]
    }
}
