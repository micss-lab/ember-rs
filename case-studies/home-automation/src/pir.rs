use esp_hal::gpio::Input;
use no_std_framework_core::{
    behaviour::{Context, TickerBehaviour},
    Agent,
};
use ontology::PirMessage;

use super::util::wrap_message;

pub fn pir_agent(sensor: Input<'static>) -> Agent<(), ()> {
    Agent::new("pir", ()).with_behaviour(Pir::new(sensor))
}

pub mod ontology {
    use no_std_framework_core::{
        acl::message::{Content, Message, Performative, Receiver},
        Aid,
    };
    use serde::{Deserialize, Serialize};

    pub struct PirOntology;

    impl PirOntology {
        pub const fn name() -> &'static str {
            "Pir-Ontology"
        }

        pub fn decode_message(message: Message) -> PirMessage {
            let Content::Bytes(content) = message.content else {
                panic!("received incorrect content type");
            };
            postcard::from_bytes(&content).expect("failed to parse content")
        }
    }

    #[derive(Serialize, Deserialize)]
    pub struct PirMessage {
        pub object_detected: bool,
    }

    impl PirMessage {
        pub(super) fn into_message(self) -> Message {
            Message {
                performative: Performative::Inform,
                sender: None,
                receiver: Receiver::Single(Aid::local("control")),
                reply_to: None,
                ontology: Some(PirOntology::name().into()),
                content: Content::Bytes(postcard::to_allocvec(&self).unwrap()),
            }
        }
    }
}

pub struct Pir {
    sensor: Input<'static>,
    object_detected: bool,
}

impl Pir {
    fn new(sensor: Input<'static>) -> Self {
        Self {
            sensor,
            object_detected: false,
        }
    }
}

impl TickerBehaviour for Pir {
    type AgentState = ();

    type Event = ();

    fn interval(&self) -> core::time::Duration {
        core::time::Duration::from_secs(1)
    }

    fn action(&mut self, ctx: &mut Context<Self::Event>, _: &mut Self::AgentState) {
        if self.object_detected != self.sensor.is_high() {
            self.object_detected = self.sensor.is_high();
            ctx.send_message(wrap_message(
                PirMessage {
                    object_detected: self.object_detected,
                }
                .into_message(),
            ));
        }
    }

    fn is_finished(&self) -> bool {
        false
    }
}
