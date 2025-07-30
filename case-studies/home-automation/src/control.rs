use no_std_framework_core::{
    acl::{
        codec::AgentActionCodec,
        message::{self, Message, Performative},
    },
    behaviour::{Context, CyclicBehaviour},
    Agent, Aid,
};

use crate::{
    dht22::ontology::Dht22Ontology,
    fan::{
        self,
        ontology::{FanAction, FanMessage},
    },
    pir::{self, ontology::PirOntology},
    util::wrap_message,
};

use super::{dht22, pir::ontology::PirMessage};

pub fn control_agent() -> Agent<(), ()> {
    Agent::new("control", ()).with_behaviour(Receiver::new())
}

struct Receiver {
    temperature: f32,
    humidity: f32,
    human_detected: bool,
    fan_powered: bool,
}

impl Receiver {
    fn new() -> Self {
        Self {
            temperature: 0.0,
            humidity: 0.0,
            human_detected: false,
            fan_powered: false,
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

        self.check_fan_state(ctx);
    }

    fn handle_pir_message(&mut self, message: PirMessage, ctx: &mut Context<()>) {
        self.human_detected = message.object_detected;

        self.check_fan_state(ctx);
    }

    fn check_fan_state(&mut self, ctx: &mut Context<()>) {
        let fan_powered_wanted = self.temperature >= 23.0 && self.human_detected;

        if self.fan_powered != fan_powered_wanted {
            self.toggle_fan(ctx);
            self.fan_powered = fan_powered_wanted;
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
                action: FanAction::Toggle,
            })),
        }))
    }
}

impl CyclicBehaviour for Receiver {
    type AgentState = ();

    type Event = ();

    fn action(&mut self, ctx: &mut Context<Self::Event>, _: &mut Self::AgentState) {
        while let Some(message) = ctx.receive_message(None) {
            let Some(ontology) = message.ontology.as_ref() else {
                unimplemented!();
            };

            if ontology == dht22::ontology::Dht22Ontology::name() {
                let measurement = Dht22Ontology::decode_message(message);
                self.handle_dht22_measurement(measurement, ctx);
            } else if ontology == pir::ontology::PirOntology::name() {
                let message = PirOntology::decode_message(message);
                self.handle_pir_message(message, ctx);
            }
        }
        ctx.block_behaviour();
    }

    fn is_finished(&self) -> bool {
        false
    }
}
