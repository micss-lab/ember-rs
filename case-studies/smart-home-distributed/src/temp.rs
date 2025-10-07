use alloc::rc::Rc;
use core::cell::RefCell;
use ontology::Temperature;

use ember::{
    Agent,
    behaviour::{Context, TickerBehaviour},
};
use esp_hal::analog::adc::{Adc, AdcChannel, AdcPin, RegisterAccess};

pub fn temperature_agent<'d, P: AdcChannel + 'd, ADCI: RegisterAccess>(
    sensor: AdcPin<P, ADCI>,
    adc: Rc<RefCell<Adc<'d, ADCI>>>,
) -> Agent<'d, (), ()> {
    Agent::new("temp", ()).with_behaviour(Sensor::new(sensor, adc))
}

pub mod ontology {
    use ember::{
        Aid,
        message::{Content, Message, Performative, Receiver},
    };
    use serde::{Deserialize, Serialize};

    pub struct TempOntology;

    #[derive(Serialize, Deserialize)]
    pub struct Temperature(pub f32);

    impl TempOntology {
        pub const fn name() -> &'static str {
            "Temp-Ontology"
        }

        pub fn decode_message(message: Message) -> Temperature {
            let Content::Bytes(content) = message.content else {
                panic!("received incorrect content type");
            };
            postcard::from_bytes(&content).expect("failed to parse content")
        }
    }

    impl Temperature {
        pub fn into_message(self) -> Message {
            Message {
                performative: Performative::Inform,
                sender: None,
                receiver: Receiver::Single(Aid::local("control")),
                reply_to: None,
                ontology: Some(TempOntology::name().into()),
                content: Content::Bytes(postcard::to_allocvec(&self).unwrap()),
            }
        }
    }
}

struct Sensor<'d, P, ADCI> {
    sensor: AdcPin<P, ADCI>,
    adc: Rc<RefCell<Adc<'d, ADCI>>>,
}

impl<'d, P, ADCI> Sensor<'d, P, ADCI> {
    fn new(sensor: AdcPin<P, ADCI>, adc: Rc<RefCell<Adc<'d, ADCI>>>) -> Self {
        Self { sensor, adc }
    }
}

impl<P, ADCI> TickerBehaviour for Sensor<'_, P, ADCI>
where
    P: AdcChannel,
    ADCI: RegisterAccess,
{
    type AgentState = ();

    type Event = ();

    fn interval(&self) -> core::time::Duration {
        core::time::Duration::from_millis(500)
    }

    fn action(&mut self, ctx: &mut Context<Self::Event>, _: &mut Self::AgentState) {
        let adc_reading = match nb::block!(self.adc.borrow_mut().read_oneshot(&mut self.sensor)) {
            Ok(r) => r,
            Err(err) => panic!("failed to read analog sensor: {:?}", err),
        };
        let temperature = adc_to_temperature(adc_reading);
        ctx.send_message(Temperature(temperature).into_message().wrap_with_envolope());
    }

    fn is_finished(&self) -> bool {
        false
    }
}

fn adc_to_temperature(adc: u16) -> f32 {
    let voltage = f32::from(adc) / 4096.0 * super::TEMP_SENSOR_VCC_VOLTAGE;
    voltage * 10.0
}
