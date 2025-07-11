use no_std_framework_core::{
    acl::{
        codec::AgentActionCodec,
        message::{self, Message, Performative},
    },
    behaviour::{Context, CyclicBehaviour},
    Agent, Aid,
};

use crate::entry::{
    fan::{
        self,
        ontology::{FanAction, FanMessage},
    },
    util::wrap_message,
};

use super::dht22;

pub fn control_agent() -> Agent<(), ()> {
    Agent::new("control", ()).with_behaviour(Receiver::new())
}

struct Receiver {
    temperature: f32,
    humidity: f32,
    human_detected: bool,
}

impl Receiver {
    fn new() -> Self {
        Self {
            temperature: 0.0,
            humidity: 0.0,
            human_detected: true,
        }
    }

    fn handle_dht22_measurement(
        &mut self,
        dht22::Measurement {
            temperature,
            humidity,
        }: dht22::Measurement,
        ctx: &mut Context<()>,
    ) {
        self.temperature = temperature;
        self.humidity = humidity;

        log::info!(
            "Current temperature and humidity: {} degrees, {} percent",
            self.temperature,
            self.humidity
        );

        if self.temperature >= 23.0 && self.human_detected {
            self.toggle_fan(ctx)
        }
    }

    fn toggle_fan(&self, ctx: &mut Context<()>) {
        ctx.send_message(wrap_message(Message {
            performative: Performative::Request,
            sender: None,
            receiver: message::Receiver::Single(Aid::local("fan")),
            reply_to: None,
            ontology: Some(fan::ontology::FanOntology::name().into()),
            content: message::Content::Sl(AgentActionCodec::into_content(FanMessage {
                fan: 0,
                action: FanAction::Toggle,
            })),
        }))
    }
}

impl CyclicBehaviour for Receiver {
    type AgentState = ();

    type Event = ();

    fn action(&mut self, ctx: &mut Context<Self::Event>, _: &mut Self::AgentState) {
        use no_std_framework_core::acl::message::Content;

        while let Some(message) = ctx.receive_message(None) {
            let Some(ontology) = message.ontology else {
                unimplemented!();
            };

            if ontology == dht22::ontology::Dht22Ontology::name() {
                let Content::Other { content, .. } = message.content else {
                    unimplemented!();
                };
                let measurement = content.parse().unwrap();

                self.handle_dht22_measurement(measurement, ctx);
            }
        }
        ctx.block_behaviour();
    }

    fn is_finished(&self) -> bool {
        false
    }
}
