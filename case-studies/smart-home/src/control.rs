use core::ptr::addr_of_mut;

use alloc::borrow::Cow;

use blocking_network_stack::Socket;
use esp_hal::gpio::{Input, Output};
use esp_wifi::wifi::{WifiDevice, WifiStaDevice};
use home_automation::{
    fan::{
        FanState,
        ontology::{FanAction, FanMessage, FanOntology},
    },
    lock::ontology::DoorLockOntology,
    pir::ontology::PirOntology,
};
use no_std_framework_core::{
    Agent,
    acl::message::MessageFilter,
    behaviour::{Context, CyclicBehaviour, TickerBehaviour},
};
use plant_monitoring::{
    light::ontology::LightOntology,
    moist::ontology::MoistureOntology,
    pump::ontology::{PumpAction, PumpOntology, PumpStatus},
};

use super::{temp::ontology::TempOntology, utils::wrap_message};

static mut RX_BUFFER: &mut [u8] = &mut [0u8; 1024];
static mut TX_BUFFER: &mut [u8] = &mut [0u8; 2048];

mod http;

pub fn control_agent(
    pump_switch: Input<'static>,
    fan_active_led: Output<'static>,
) -> Agent<HomeData, ()> {
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
        // .with_behaviour(DataPrinter)
        .with_behaviour(HttpServer::new(super::HTTP_SERVER_PORT))
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
            MoistureOntology::name().into(),
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
            LightOntology::name().into(),
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
            TempOntology::name().into(),
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
            PumpOntology::name().into(),
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

struct PumpControl {
    pump_switch: Input<'static>,
}

impl PumpControl {
    fn new(pump_switch: Input<'static>) -> Self {
        Self { pump_switch }
    }
}

impl TickerBehaviour for PumpControl {
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
            PirOntology::name().into(),
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

struct FanStateReceiver {
    fan_active_led: Output<'static>,
}

impl FanStateReceiver {
    fn new(fan_active_led: Output<'static>) -> Self {
        Self { fan_active_led }
    }
}

impl TickerBehaviour for FanStateReceiver {
    type AgentState = HomeData;

    type Event = ();

    fn interval(&self) -> core::time::Duration {
        core::time::Duration::from_millis(750)
    }

    fn action(&mut self, ctx: &mut Context<Self::Event>, state: &mut Self::AgentState) {
        let Some(message) = ctx.receive_message(Some(Cow::Owned(MessageFilter::ontology(
            FanOntology::name().into(),
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
            DoorLockOntology::name().into(),
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

struct HttpServer {
    port: u16,
    current_socket: Option<Socket<'static, 'static, 'static, WifiDevice<'static, WifiStaDevice>>>,
}

impl HttpServer {
    fn new(http_port: u16) -> Self {
        log::trace!("started http server");
        Self {
            port: http_port,
            current_socket: None,
        }
    }
}

impl CyclicBehaviour for HttpServer {
    type AgentState = HomeData;

    type Event = ();

    fn action(&mut self, _: &mut Context<Self::Event>, state: &mut Self::AgentState) {
        use embedded_io::{Read, Write};

        let mut socket = self.current_socket.take().unwrap_or_else(|| {
            log::trace!("Waiting for socket");
            let mut socket = unsafe { &mut *addr_of_mut!(crate::WIFI_STACK) }
                .get()
                .unwrap()
                .get_socket(unsafe { *addr_of_mut!(RX_BUFFER) }, unsafe {
                    *addr_of_mut!(TX_BUFFER)
                });
            socket.listen_unblocking(self.port).unwrap();
            socket
        });

        if !socket.is_connected() {
            socket.work();
            self.current_socket = Some(socket);
            return;
        }

        log::trace!("Incoming connection.");

        let mut buf = [0u8; 4096];

        if socket.read(&mut buf).is_err() {
            return;
        }

        let mut headers = [httparse::EMPTY_HEADER; 16];
        let mut req = httparse::Request::new(&mut headers);
        if let Err(err) = req.parse(&buf) {
            log::error!("Error parsing incoming request: {}", err);
            socket.close();
            return;
        };

        log::debug!("Incoming request: {:?}", req);

        let (status, body) = http::handle_request(req, state);
        http::write_response(&mut socket, status, body);

        log::trace!("Closing socket.");
        let _ = socket.flush();
        socket.close();
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
        core::time::Duration::from_secs(3)
    }

    fn action(&mut self, ctx: &mut Context<Self::Event>, _: &mut Self::AgentState) {
        while ctx.receive_message(None).is_some() {}
    }

    fn is_finished(&self) -> bool {
        false
    }
}
