use alloc::format;

use ember::{
    Agent, Aid,
    behaviour::{Context, TickerBehaviour},
    message::{Message, Performative, Receiver},
};

pub fn dht22_agent(
    measurements: impl IntoIterator<Item = Measurement> + 'static,
) -> Agent<'static, (), ()> {
    Agent::new("dht22", ()).with_behaviour(Sensor::new(measurements.into_iter()))
}

pub mod ontology {
    use ember::message::{Content, Message};

    use super::Measurement;

    pub struct Dht22Ontology;

    impl Dht22Ontology {
        pub const fn name() -> &'static str {
            "Dht22-Ontology"
        }

        pub fn decode_message(message: Message) -> Measurement {
            let Content::Other { content, .. } = message.content else {
                panic!("received incorrect content type");
            };
            content.parse().expect("failed to parse message content")
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Measurement {
    pub temperature: f32,
    pub humidity: f32,
}

impl core::str::FromStr for Measurement {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (temperature, humidity) = {
            let (t, h) = s.split_once(',').ok_or(())?;
            (t.parse().map_err(|_| ())?, h.parse().map_err(|_| ())?)
        };

        Ok(Self {
            temperature,
            humidity,
        })
    }
}

impl Measurement {
    fn into_message(self) -> Message {
        use ember::message::Content;

        Message {
            performative: Performative::Inform,
            sender: None,
            receiver: Receiver::Single(Aid::local("control")),
            reply_to: None,
            ontology: Some(ontology::Dht22Ontology::name().into()),
            content: Content::Other {
                kind: None,
                content: format!("{},{}", self.temperature, self.humidity),
            },
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
