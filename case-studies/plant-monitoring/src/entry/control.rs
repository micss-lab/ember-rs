use no_std_framework_core::{
    behaviour::{Context, CyclicBehaviour},
    Agent,
};

use super::temp;

pub fn control_agent() -> Agent<(), ()> {
    Agent::new("control", ()).with_behaviour(Receiver::new())
}

struct Receiver {
    temperature: f32,
    humidity: f32,
}

impl Receiver {
    fn new() -> Self {
        Self {
            temperature: 0.0,
            humidity: 0.0,
        }
    }

    fn handle_temp_measurement(
        &mut self,
        temp::Measurement {
            temperature,
            humidity,
        }: temp::Measurement,
        _: &mut Context<()>,
    ) {
        self.temperature = temperature;
        self.humidity = humidity;

        log::info!(
            "Current temperature and humidity: {} degrees, {} percent",
            self.temperature,
            self.humidity
        );
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

            if ontology == temp::ontology::TempOntology::name() {
                let measurement = temp::ontology::TempOntology::decode_message(message).unwrap();
                self.handle_temp_measurement(measurement, ctx);
            }
        }
        ctx.block_behaviour();
    }

    fn is_finished(&self) -> bool {
        false
    }
}
