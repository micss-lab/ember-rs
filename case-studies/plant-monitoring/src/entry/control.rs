use alloc::borrow::Cow;
use esp_hal::gpio::Input;
use no_std_framework_core::{
    acl::message::MessageFilter,
    behaviour::{Context, CyclicBehaviour, TickerBehaviour},
    Agent,
};

use super::{
    light, moist,
    pump::{
        self,
        ontology::{PumpAction, PumpStatus},
    },
    temp,
    util::wrap_message,
};

pub fn control_agent(user_switch: Input<'static>) -> Agent<ControlState, ()> {
    Agent::new("control", ControlState::default())
        .with_behaviour(Receiver)
        .with_behaviour(PumpControl::new(user_switch))
        .with_behaviour(StatusPrinter)
}

#[derive(Default)]
pub struct ControlState {
    temperature: f32,
    humidity: f32,
    moisture: f32,
    pump_active: bool,
    light: f32,
}

impl ControlState {
    fn handle_temp_measurement(
        &mut self,
        temp::Measurement {
            temperature,
            humidity,
        }: temp::Measurement,
    ) {
        self.temperature = temperature;
        self.humidity = humidity;

        log::info!(
            "Current temperature and humidity: {} degrees, {} percent",
            self.temperature,
            self.humidity
        );
    }

    fn handle_moisture_measurement(&mut self, percent: f32) {
        self.moisture = percent;
    }

    fn handle_light_level(&mut self, level: f32) {
        self.light = level;
    }

    fn handle_pump_status(&mut self, status: PumpStatus) {
        if !status.changed {
            log::warn!("Pump already in requested state.");
        } else {
            if status.active {
                log::info!("Pump successfully activated!");
            } else {
                log::info!("Pump successfully deactivated!");
            }
        }

        self.pump_active = status.active;
    }
}

struct PumpControl {
    user_switch: Input<'static>,
}

impl PumpControl {
    fn new(user_switch: Input<'static>) -> Self {
        Self { user_switch }
    }
}

impl TickerBehaviour for PumpControl {
    type AgentState = ControlState;

    type Event = ();

    fn interval(&self) -> core::time::Duration {
        core::time::Duration::from_millis(250)
    }

    fn action(&mut self, ctx: &mut Context<Self::Event>, state: &mut Self::AgentState) {
        let pump_should_be_active =
            self.user_switch.is_low() || state.moisture < super::MOISTURE_THRESHOLD;
        let action = match (pump_should_be_active, state.pump_active) {
            (true, false) => PumpAction::Activate,
            (false, true) => PumpAction::Deactivate,
            _ => return,
        };

        ctx.send_message(wrap_message(action.into_message()));
    }

    fn is_finished(&self) -> bool {
        false
    }
}

struct Receiver;

impl CyclicBehaviour for Receiver {
    type AgentState = ControlState;

    type Event = ();

    fn action(&mut self, ctx: &mut Context<Self::Event>, state: &mut Self::AgentState) {
        let mut received = false;

        while let Some(message) = ctx.receive_message(temp_ontology_message_filter()) {
            received = true;
            let measurement = temp::ontology::TempOntology::decode_message(message).unwrap();
            state.handle_temp_measurement(measurement);
        }

        while let Some(message) = ctx.receive_message(moisture_ontology_message_filter()) {
            received = true;
            let moisture_percent =
                moist::ontology::MoistureOntology::decode_message(message).unwrap();
            state.handle_moisture_measurement(moisture_percent.0);
        }

        while let Some(message) = ctx.receive_message(light_ontology_message_filter()) {
            received = true;
            let light_level = light::ontology::LightOntology::decode_message(message).unwrap();
            state.handle_light_level(light_level.0);
        }

        while let Some(message) = ctx.receive_message(pump_ontology_message_filter()) {
            received = true;
            let pump_status = pump::ontology::PumpOntology::decode_message(message).unwrap();
            state.handle_pump_status(pump_status);
        }

        if !received {
            ctx.block_behaviour();
        }
    }

    fn is_finished(&self) -> bool {
        false
    }
}

fn temp_ontology_message_filter() -> Option<Cow<'static, MessageFilter>> {
    Some(Cow::Owned(MessageFilter::ontology(
        temp::ontology::TempOntology::name().into(),
    )))
}

fn moisture_ontology_message_filter() -> Option<Cow<'static, MessageFilter>> {
    Some(Cow::Owned(MessageFilter::ontology(
        moist::ontology::MoistureOntology::name().into(),
    )))
}

fn light_ontology_message_filter() -> Option<Cow<'static, MessageFilter>> {
    Some(Cow::Owned(MessageFilter::ontology(
        light::ontology::LightOntology::name().into(),
    )))
}

fn pump_ontology_message_filter() -> Option<Cow<'static, MessageFilter>> {
    Some(Cow::Owned(MessageFilter::ontology(
        pump::ontology::PumpOntology::name().into(),
    )))
}

struct StatusPrinter;

impl TickerBehaviour for StatusPrinter {
    type AgentState = ControlState;

    type Event = ();

    fn interval(&self) -> core::time::Duration {
        core::time::Duration::from_secs(1)
    }

    fn action(&mut self, _: &mut Context<Self::Event>, state: &mut Self::AgentState) {
        log::info!("-----------------------------");
        log::info!("Temperature: {}", state.temperature);
        log::info!("Humidity: {}", state.humidity);
        log::info!("Light: {}", state.light);
        log::info!("Moisture: {}", state.moisture);
        log::info!("Pump Active: {}", state.pump_active);
        log::info!("-----------------------------");
    }

    fn is_finished(&self) -> bool {
        false
    }
}
