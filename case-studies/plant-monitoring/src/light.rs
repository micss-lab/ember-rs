use core::cell::RefCell;

use alloc::{borrow::Cow, rc::Rc};
use esp_hal::{
    analog::adc::{Adc, AdcChannel, AdcPin, RegisterAccess},
    gpio::Output,
};
use ember::{
    Agent,
    behaviour::{Context, TickerBehaviour},
};
use ontology::LightLevel;

use super::{
    notif::{ThresholdConfig, ThresholdNotification},
    util::wrap_message,
};

pub fn light_agent<P: AdcChannel + 'static, ADCI: RegisterAccess>(
    ldr_sensor_pin: AdcPin<P, ADCI>,
    adc: Rc<RefCell<Adc<'static, ADCI>>>,
    light_alert_pin: Output<'static>,
) -> Agent<LightState, ()> {
    Agent::new("ldr", LightState::default())
        .with_behaviour(LdrSensor::new(ldr_sensor_pin, adc))
        .with_behaviour(LightAlert(light_alert_pin))
        .with_behaviour(ThresholdNotification::new())
}

#[derive(Default)]
pub struct LightState {
    lux: f32,
}

impl ThresholdConfig for LightState {
    fn current(&self) -> f32 {
        self.lux
    }

    fn low(&self) -> f32 {
        super::LIGHT_LOW_THRESHOLD
    }

    fn low_notification(&self, _: f32) -> Cow<'static, str> {
        "🌑 Low Light Alert - Too dark for plants!".into()
    }

    fn high(&self) -> f32 {
        super::LIGHT_HIGH_THRESHOLD
    }

    fn high_notification(&self, _: f32) -> Cow<'static, str> {
        "☀️ High Light Alert - Too much sunlight!".into()
    }

    fn normalized_notification(&self, _: f32) -> Cow<'static, str> {
        "Light normalised".into()
    }
}

pub mod ontology {
    use ember::{
        Aid,
        message::{Content, Message, Performative, Receiver},
    };
    use serde::{Deserialize, Serialize};

    pub struct LightOntology;

    #[derive(Serialize, Deserialize)]
    pub struct LightLevel(pub f32);

    impl LightOntology {
        pub const fn name() -> &'static str {
            "Light-Ontology"
        }

        pub fn decode_message(message: Message) -> LightLevel {
            let Content::Bytes(content) = message.content else {
                panic!("receiverd incorrect content type")
            };
            postcard::from_bytes(&content).expect("failed to parse message content")
        }
    }

    impl LightLevel {
        pub fn into_message(self) -> Message {
            Message {
                performative: Performative::Inform,
                sender: None,
                receiver: Receiver::Single(Aid::local("control")),
                reply_to: None,
                ontology: Some(LightOntology::name().into()),
                content: Content::Bytes(postcard::to_allocvec(&self).unwrap()),
            }
        }
    }
}

struct LdrSensor<P, ADCI: 'static> {
    pin: AdcPin<P, ADCI>,
    adc: Rc<RefCell<Adc<'static, ADCI>>>,
}

impl<P, ADCI> LdrSensor<P, ADCI> {
    fn new(pin: AdcPin<P, ADCI>, adc: Rc<RefCell<Adc<'static, ADCI>>>) -> Self {
        Self { pin, adc }
    }
}

impl<P, ADCI> TickerBehaviour for LdrSensor<P, ADCI>
where
    P: AdcChannel,
    ADCI: RegisterAccess,
{
    type AgentState = LightState;

    type Event = ();

    fn interval(&self) -> core::time::Duration {
        core::time::Duration::from_millis(100)
    }

    fn action(&mut self, ctx: &mut Context<Self::Event>, state: &mut Self::AgentState) {
        let adc_reading = match nb::block!(self.adc.borrow_mut().read_oneshot(&mut self.pin)) {
            Ok(r) => r,
            Err(err) => panic!("failed to read analog sensor: {:?}", err),
        };

        let lux = adc_to_lux(adc_reading);
        state.lux = lux;
        ctx.send_message(wrap_message(LightLevel(lux).into_message()));
    }

    fn is_finished(&self) -> bool {
        false
    }
}

struct LightAlert(Output<'static>);

impl TickerBehaviour for LightAlert {
    type AgentState = LightState;

    type Event = ();

    fn interval(&self) -> core::time::Duration {
        core::time::Duration::from_millis(250)
    }

    fn action(&mut self, _: &mut Context<Self::Event>, state: &mut Self::AgentState) {
        self.0
            .set_level((state.lux < super::LIGHT_ALERT_THRESHOLD).into())
    }

    fn is_finished(&self) -> bool {
        false
    }
}

fn adc_to_lux(adc: u16) -> f32 {
    let adc = (f32::from(adc) + super::LDR_ADC_RANGE_OFFSET).clamp(0.0, 4096.0);
    let voltage = adc / 4096.0 * super::LDR_VCC_VOLTAGE;
    let resistance = 2000.0 * voltage / (1.0 - voltage / super::LDR_VCC_VOLTAGE);
    libm::powf(
        super::LDR_RL10 * 1e3_f32 * libm::powf(10.0, super::LDR_GAMMA) / resistance,
        1.0 / super::LDR_GAMMA,
    )
}
