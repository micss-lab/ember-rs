use core::cell::RefCell;

use alloc::{borrow::Cow, rc::Rc};
use ember::{
    Agent,
    behaviour::{Context, TickerBehaviour},
};
use esp_hal::analog::adc::{Adc, AdcChannel, AdcPin, RegisterAccess};
use ontology::MoisturePercent;

use super::notif::{ThresholdConfig, ThresholdNotification};

pub fn moisture_agent<'d, P: AdcChannel + 'd, ADCI: RegisterAccess>(
    potentiometer_sensor_pin: AdcPin<P, ADCI>,
    adc: Rc<RefCell<Adc<'d, ADCI>>>,
) -> Agent<'d, MoistureState, ()> {
    Agent::new("moisture", MoistureState::default())
        .with_behaviour(PotentiometerSensor::new(potentiometer_sensor_pin, adc))
        .with_behaviour(ThresholdNotification::new())
}

#[derive(Default)]
pub struct MoistureState {
    percent: f32,
}

impl ThresholdConfig for MoistureState {
    fn current(&self) -> f32 {
        self.percent
    }

    fn low(&self) -> f32 {
        super::MOISTURE_LOW_THRESHOLD
    }

    fn low_notification(&self, _: f32) -> Cow<'static, str> {
        "🌱 Dry Soil Alert - Water your plant!".into()
    }

    fn high(&self) -> f32 {
        super::MOISTURE_HIGH_THRESHOLD
    }

    fn high_notification(&self, _: f32) -> Cow<'static, str> {
        "💦 Wet Soil Alert - Too much water!".into()
    }

    fn normalized_notification(&self, _: f32) -> Cow<'static, str> {
        "Moisture normalised".into()
    }
}

pub mod ontology {
    use ember::{
        Aid,
        message::{Content, Message, Performative, Receiver},
    };
    use serde::{Deserialize, Serialize};

    pub struct MoistureOntology;

    #[derive(Serialize, Deserialize)]
    pub struct MoisturePercent(pub f32);

    impl MoistureOntology {
        pub const fn name() -> &'static str {
            "Moisture-Ontology"
        }

        pub fn decode_message(message: Message) -> MoisturePercent {
            let Content::Bytes(content) = message.content else {
                panic!("received incorrect content type");
            };
            postcard::from_bytes(&content).expect("failed to parse content")
        }
    }

    impl MoisturePercent {
        pub fn into_message(self) -> Message {
            Message {
                performative: Performative::Inform,
                sender: None,
                receiver: Receiver::Single(Aid::local("control")),
                reply_to: None,
                ontology: Some(MoistureOntology::name().into()),
                content: Content::Bytes(postcard::to_allocvec(&self).unwrap()),
            }
        }
    }
}

struct PotentiometerSensor<'d, P, ADCI> {
    pin: AdcPin<P, ADCI>,
    adc: Rc<RefCell<Adc<'d, ADCI>>>,
}

impl<'d, P, ADCI> PotentiometerSensor<'d, P, ADCI> {
    fn new(pin: AdcPin<P, ADCI>, adc: Rc<RefCell<Adc<'d, ADCI>>>) -> Self {
        Self { pin, adc }
    }
}

impl<P, ADCI> TickerBehaviour for PotentiometerSensor<'_, P, ADCI>
where
    P: AdcChannel,
    ADCI: RegisterAccess,
{
    type AgentState = MoistureState;

    type Event = ();

    fn interval(&self) -> core::time::Duration {
        core::time::Duration::from_millis(100)
    }

    fn action(&mut self, ctx: &mut Context<Self::Event>, state: &mut Self::AgentState) {
        let moisture = match nb::block!(self.adc.borrow_mut().read_oneshot(&mut self.pin)) {
            Ok(r) => r,
            Err(err) => panic!("failed to read analog sensor: {:?}", err),
        };
        let percent = f32::from(moisture) / 4095.0 * 100.0;
        state.percent = percent;
        ctx.send_message(MoisturePercent(percent).into_message().wrap_with_envolope())
    }

    fn is_finished(&self) -> bool {
        false
    }
}
