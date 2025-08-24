use alloc::borrow::Cow;

use blocking_network_stack::Stack;
use ember::{
    Agent,
    behaviour::{Context, TickerBehaviour},
    message::MessageFilter,
};
use esp_hal::gpio::{Input, Output};
use home_automation::{
    fan::{
        FanState,
        ontology::{FanAction, FanMessage, FanOntology},
    },
    lock::ontology::DoorLockOntology,
    pir::ontology::PirOntology,
};
use http_server::http::Server;
use plant_monitoring::{
    light::ontology::LightOntology,
    moist::ontology::MoistureOntology,
    pump::ontology::{PumpAction, PumpOntology, PumpStatus},
};
use smoltcp::phy::Device;

use super::{temp::ontology::TempOntology, utils::wrap_message};

mod http;

pub fn control_agent<'a, D: Device>(
    stack: &'a Stack<'static, D>,
    pump_switch: Input<'a>,
    fan_active_led: Output<'a>,
) -> Agent<'a, HomeData, ()> {
    Agent::new("control", HomeData::default())
        .with_behaviour(MoistureReceiver)
        .with_behaviour(LightLevelReceiver)
        .with_behaviour(TemperatureReceiver)
        .with_behaviour(PumpStateReceiver)
        .with_behaviour(PumpControl::new(pump_switch))
        .with_behaviour(HumanDetectedReceiver)
        .with_behaviour(DoorLockActionReceiver)
        .with_behaviour(FanStateReceiver::new(fan_active_led))
        .with_behaviour(FanControl)
        .with_behaviour(DataPrinter)
        .with_behaviour(Server::new(
            super::HTTP_SERVER_PORT,
            http::handle_request,
            stack,
        ))
        .with_behaviour(Trunk)
}

#[derive(Default, Clone)]
pub struct HomeData {
    moisture: f32,
    light_level: f32,
    temperature: f32,
    pump_active: bool,
    door_locked: bool,
    fan_active: bool,
    human_home: bool,
}

struct MoistureReceiver;

impl TickerBehaviour for MoistureReceiver {
    type AgentState = HomeData;

    type Event = ();

    fn interval(&self) -> core::time::Duration {
        core::time::Duration::from_millis(100)
    }

    fn action(&mut self, ctx: &mut Context<Self::Event>, state: &mut Self::AgentState) {
        let Some(message) = ctx.receive_message(Some(Cow::Owned(MessageFilter::ontology(
            MoistureOntology::name(),
        )))) else {
            ctx.block_behaviour();
            return;
        };

        state.moisture = MoistureOntology::decode_message(message).0;
    }

    fn is_finished(&self) -> bool {
        false
    }
}

struct LightLevelReceiver;

impl TickerBehaviour for LightLevelReceiver {
    type AgentState = HomeData;

    type Event = ();

    fn interval(&self) -> core::time::Duration {
        core::time::Duration::from_millis(100)
    }

    fn action(&mut self, ctx: &mut Context<Self::Event>, state: &mut Self::AgentState) {
        let Some(message) = ctx.receive_message(Some(Cow::Owned(MessageFilter::ontology(
            LightOntology::name(),
        )))) else {
            ctx.block_behaviour();
            return;
        };
        state.light_level = LightOntology::decode_message(message).0;
    }

    fn is_finished(&self) -> bool {
        false
    }
}

struct TemperatureReceiver;

impl TickerBehaviour for TemperatureReceiver {
    type AgentState = HomeData;

    type Event = ();

    fn interval(&self) -> core::time::Duration {
        core::time::Duration::from_millis(100)
    }

    fn action(&mut self, ctx: &mut Context<Self::Event>, state: &mut Self::AgentState) {
        let Some(message) = ctx.receive_message(Some(Cow::Owned(MessageFilter::ontology(
            TempOntology::name(),
        )))) else {
            ctx.block_behaviour();
            return;
        };
        state.temperature = TempOntology::decode_message(message).0;
    }

    fn is_finished(&self) -> bool {
        false
    }
}

struct PumpStateReceiver;

impl TickerBehaviour for PumpStateReceiver {
    type AgentState = HomeData;

    type Event = ();

    fn interval(&self) -> core::time::Duration {
        core::time::Duration::from_millis(100)
    }

    fn action(&mut self, ctx: &mut Context<Self::Event>, state: &mut Self::AgentState) {
        let Some(message) = ctx.receive_message(Some(Cow::Owned(MessageFilter::ontology(
            PumpOntology::name(),
        )))) else {
            ctx.block_behaviour();
            return;
        };
        state.pump_active = PumpOntology::decode_message::<PumpStatus>(message).active;
    }

    fn is_finished(&self) -> bool {
        false
    }
}

struct PumpControl<'d> {
    pump_switch: Input<'d>,
}

impl<'d> PumpControl<'d> {
    fn new(pump_switch: Input<'d>) -> Self {
        Self { pump_switch }
    }
}

impl TickerBehaviour for PumpControl<'_> {
    type AgentState = HomeData;

    type Event = ();

    fn interval(&self) -> core::time::Duration {
        core::time::Duration::from_millis(250)
    }

    fn action(&mut self, ctx: &mut Context<Self::Event>, state: &mut Self::AgentState) {
        let pump_should_be_active =
            self.pump_switch.is_low() || state.moisture < super::MOISTURE_THRESHOLD;
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

struct HumanDetectedReceiver;

impl TickerBehaviour for HumanDetectedReceiver {
    type AgentState = HomeData;

    type Event = ();

    fn interval(&self) -> core::time::Duration {
        core::time::Duration::from_millis(100)
    }

    fn action(&mut self, ctx: &mut Context<Self::Event>, state: &mut Self::AgentState) {
        let Some(message) = ctx.receive_message(Some(Cow::Owned(MessageFilter::ontology(
            PirOntology::name(),
        )))) else {
            ctx.block_behaviour();
            return;
        };
        state.human_home = PirOntology::decode_message(message).object_detected;
    }

    fn is_finished(&self) -> bool {
        false
    }
}

struct FanStateReceiver<'d> {
    fan_active_led: Output<'d>,
}

impl<'d> FanStateReceiver<'d> {
    fn new(fan_active_led: Output<'d>) -> Self {
        Self { fan_active_led }
    }
}

impl TickerBehaviour for FanStateReceiver<'_> {
    type AgentState = HomeData;

    type Event = ();

    fn interval(&self) -> core::time::Duration {
        core::time::Duration::from_millis(750)
    }

    fn action(&mut self, ctx: &mut Context<Self::Event>, state: &mut Self::AgentState) {
        let Some(message) = ctx.receive_message(Some(Cow::Owned(MessageFilter::ontology(
            FanOntology::name(),
        )))) else {
            ctx.block_behaviour();
            return;
        };
        log::debug!("Receiving state from fan");
        state.fan_active = match FanOntology::decode_message(message) {
            FanState::On => true,
            FanState::Off => false,
        };
        self.fan_active_led.set_level(state.fan_active.into())
    }

    fn is_finished(&self) -> bool {
        false
    }
}

struct FanControl;

impl TickerBehaviour for FanControl {
    type AgentState = HomeData;

    type Event = ();

    fn interval(&self) -> core::time::Duration {
        core::time::Duration::from_secs(3)
    }

    fn action(&mut self, ctx: &mut Context<Self::Event>, state: &mut Self::AgentState) {
        let should_fan_activate =
            state.temperature >= super::FAN_TEMPERATURE_THRESHOLD && state.human_home;

        if state.fan_active != should_fan_activate {
            ctx.send_message(wrap_message(
                FanMessage {
                    action: FanAction::Toggle,
                }
                .into_message(),
            ));
        }
    }

    fn is_finished(&self) -> bool {
        false
    }
}

struct DoorLockActionReceiver;

impl TickerBehaviour for DoorLockActionReceiver {
    type AgentState = HomeData;

    type Event = ();

    fn interval(&self) -> core::time::Duration {
        core::time::Duration::from_millis(250)
    }

    fn action(&mut self, ctx: &mut Context<Self::Event>, state: &mut Self::AgentState) {
        let Some(message) = ctx.receive_message(Some(Cow::Owned(MessageFilter::ontology(
            DoorLockOntology::name(),
        )))) else {
            ctx.block_behaviour();
            return;
        };
        state.door_locked = DoorLockOntology::decode_message(message).locked();
    }

    fn is_finished(&self) -> bool {
        false
    }
}

#[allow(unused)]
struct DataPrinter;

impl TickerBehaviour for DataPrinter {
    type AgentState = HomeData;

    type Event = ();

    fn interval(&self) -> core::time::Duration {
        core::time::Duration::from_secs(1)
    }

    fn action(&mut self, _: &mut Context<Self::Event>, state: &mut Self::AgentState) {
        log::debug!("-------------------------------------");
        log::debug!("Home sensor data:");
        log::debug!("Moisture: {}", state.moisture);
        log::debug!("Light level: {}", state.light_level);
        log::debug!("Temperature: {}", state.temperature);
        log::debug!("Pump active: {}", state.pump_active);
        log::debug!("Human home: {}", state.human_home);
        log::debug!("Door locked: {}", state.door_locked);
        log::debug!("Fan active: {}", state.fan_active);
        log::debug!("-------------------------------------");
    }

    fn is_finished(&self) -> bool {
        false
    }
}

struct Trunk;

impl TickerBehaviour for Trunk {
    type AgentState = HomeData;

    type Event = ();

    fn interval(&self) -> core::time::Duration {
        core::time::Duration::from_secs(1)
    }

    fn action(&mut self, ctx: &mut Context<Self::Event>, _: &mut Self::AgentState) {
        while ctx.receive_message(None).is_some() {}
    }

    fn is_finished(&self) -> bool {
        false
    }
}
