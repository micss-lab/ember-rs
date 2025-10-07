use ember::{
    Agent, Aid,
    behaviour::{Context, TickerBehaviour},
    message::{Message, Performative, Receiver},
};
use serde::{Deserialize, Serialize};

pub fn temperature_agent<'a>(
    measurements: impl IntoIterator<Item = Measurement> + 'a,
) -> Agent<'a, (), ()> {
    Agent::new("temperature", ()).with_behaviour(Sensor::new(measurements.into_iter()))
}

pub mod ontology {
    use ember::message::{Content, Message};

    use super::Measurement;

    pub struct TempOntology;

    impl TempOntology {
        pub const fn name() -> &'static str {
            "Temp-Ontology"
        }

        pub fn decode_message(message: Message) -> Measurement {
            let Content::Bytes(content) = message.content else {
                panic!("received incorrect content type");
            };
            postcard::from_bytes(&content).expect("failed to parse content")
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
        use ember::message::Content;

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
        ctx.send_message(measurement.into_message().wrap_with_envolope())
    }

    fn is_finished(&self) -> bool {
        self.is_empty
    }
}
