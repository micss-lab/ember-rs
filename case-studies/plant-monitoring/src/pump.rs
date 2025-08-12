use ember::{
    Agent,
    behaviour::{Context, CyclicBehaviour},
};
use esp_hal::gpio::Output;
use ontology::{PumpAction, PumpStatus};

use super::util::wrap_message;

pub fn pump_agent(pump_light: Output<'static>) -> Agent<PumpState, ()> {
    Agent::new("pump", PumpState::default())
        .with_behaviour(PumpInteractions)
        .with_behaviour(PumpLight(pump_light))
}

#[derive(Default)]
pub struct PumpState {
    active: bool,
}

pub mod ontology {
    use ember::{
        Aid,
        message::{Content, Message, Performative, Receiver},
    };
    use serde::{Deserialize, Serialize};

    pub struct PumpOntology;

    #[derive(Debug, Deserialize, Serialize)]
    pub enum PumpAction {
        Activate,
        Deactivate,
    }

    #[derive(Deserialize, Serialize)]
    pub struct PumpStatus {
        pub active: bool,
        pub changed: bool,
    }

    impl PumpOntology {
        pub const fn name() -> &'static str {
            "Pump-Ontology"
        }

        pub fn decode_message<T>(message: Message) -> T
        where
            T: for<'de> Deserialize<'de>,
        {
            let Content::Bytes(content) = message.content else {
                panic!("received incorrect content type");
            };
            postcard::from_bytes(&content).expect("failed to parse content")
        }
    }

    impl PumpAction {
        pub fn into_message(self) -> Message {
            Message {
                performative: Performative::Request,
                sender: None,
                receiver: Receiver::Single(Aid::local("pump")),
                reply_to: None,
                ontology: Some(PumpOntology::name().into()),
                content: Content::Bytes(postcard::to_allocvec(&self).unwrap()),
            }
        }
    }

    impl PumpStatus {
        pub fn into_message(self) -> Message {
            Message {
                performative: Performative::Inform,
                sender: None,
                receiver: Receiver::Single(Aid::local("control")),
                reply_to: None,
                ontology: Some(PumpOntology::name().into()),
                content: Content::Bytes(postcard::to_allocvec(&self).unwrap()),
            }
        }
    }
}

struct PumpInteractions;

impl CyclicBehaviour for PumpInteractions {
    type AgentState = PumpState;

    type Event = ();

    fn action(&mut self, ctx: &mut Context<Self::Event>, state: &mut Self::AgentState) {
        let Some(message) = ctx.receive_message(None) else {
            ctx.block_behaviour();
            return;
        };

        let action = ontology::PumpOntology::decode_message(message);
        match (action, state.active) {
            (PumpAction::Activate, false) => state.active = true,
            (PumpAction::Deactivate, true) => state.active = false,
            _ => {
                ctx.send_message(wrap_message(
                    PumpStatus {
                        active: state.active,
                        changed: false,
                    }
                    .into_message(),
                ));
                return;
            }
        };
        log::debug!("new pump state: {}", state.active);
        ctx.send_message(wrap_message(
            PumpStatus {
                active: state.active,
                changed: true,
            }
            .into_message(),
        ));
    }

    fn is_finished(&self) -> bool {
        false
    }
}

struct PumpLight(Output<'static>);

impl CyclicBehaviour for PumpLight {
    type AgentState = PumpState;

    type Event = ();

    fn action(&mut self, _: &mut Context<Self::Event>, state: &mut Self::AgentState) {
        self.0.set_level(state.active.into());
    }

    fn is_finished(&self) -> bool {
        false
    }
}
