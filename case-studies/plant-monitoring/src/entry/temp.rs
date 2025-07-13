use alloc::format;

use no_std_framework_core::{
    acl::message::{Message, Performative, Receiver},
    behaviour::{Context, TickerBehaviour},
    Agent, Aid,
};
use serde::{Deserialize, Serialize};

use super::util::wrap_message;

pub fn temperature_agent(
    measurements: impl IntoIterator<Item = Measurement> + 'static,
) -> Agent<(), ()> {
    Agent::new("temperature", ()).with_behaviour(Sensor::new(measurements.into_iter()))
}

pub mod ontology {
    use no_std_framework_core::acl::message::{Content, Message};

    use super::Measurement;

    pub struct TempOntology;

    impl TempOntology {
        pub const fn name() -> &'static str {
            "Temp-Ontology"
        }

        pub fn decode_message(message: Message) -> Result<Measurement, ()> {
            let Content::Bytes(content) = message.content else {
                return Err(());
            };
            postcard::from_bytes(&content).map_err(|_| ())
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Measurement {
    pub temperature: f32,
    pub humidity: f32,
}

impl Measurement {
    fn into_message(self) -> Message {
        use no_std_framework_core::acl::message::Content;

        Message {
            performative: Performative::Inform,
            sender: None,
            receiver: Receiver::Single(Aid::local("control")),
            reply_to: None,
            ontology: Some(ontology::TempOntology::name().into()),
            content: Content::Bytes(postcard::to_allocvec(&self).unwrap()),
        }
    }
}

pub struct Sensor<M> {
    measurements: M,
    is_empty: bool,
}

impl<M> Sensor<M> {
    pub fn new(measurements: M) -> Self {
        Self {
            measurements,
            is_empty: false,
        }
    }
}

impl<M> TickerBehaviour for Sensor<M>
where
    M: Iterator<Item = Measurement>,
{
    type AgentState = ();

    type Event = ();

    fn interval(&self) -> core::time::Duration {
        core::time::Duration::from_secs(3)
    }

    fn action(&mut self, ctx: &mut Context<Self::Event>, _: &mut Self::AgentState) {
        let Some(measurement) = self.measurements.next() else {
            self.is_empty = true;
            return;
        };
        ctx.send_message(wrap_message(measurement.into_message()))
    }

    fn is_finished(&self) -> bool {
        self.is_empty
    }
}
