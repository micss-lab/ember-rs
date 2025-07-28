use alloc::rc::Rc;
use core::cell::RefCell;

use esp_hal::analog::adc::{Adc, AdcChannel, AdcPin, RegisterAccess};
use no_std_framework_core::{
    behaviour::{Context, TickerBehaviour},
    Agent,
};

pub fn temperature_agent<P: AdcChannel + 'static, ADCI: RegisterAccess + 'static>(
    sensor: AdcPin<P, ADCI>,
    adc: Rc<RefCell<Adc<'static, ADCI>>>,
) -> Agent<(), ()> {
    Agent::new("temp", ()).with_behaviour(Sensor::new(sensor, adc))
}

struct Sensor<P, ADCI: 'static> {
    sensor: AdcPin<P, ADCI>,
    adc: Rc<RefCell<Adc<'static, ADCI>>>,
}

impl<P, ADCI: 'static> Sensor<P, ADCI> {
    fn new(sensor: AdcPin<P, ADCI>, adc: Rc<RefCell<Adc<'static, ADCI>>>) -> Self {
        Self { sensor, adc }
    }
}

impl<P, ADCI: 'static> TickerBehaviour for Sensor<P, ADCI>
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
        let adc_reading = loop {
            match self.adc.borrow_mut().read_oneshot(&mut self.sensor) {
                Ok(r) => break r,
                Err(esp_hal::prelude::nb::Error::WouldBlock) => continue,
                Err(err) => panic!("failed to read analog sensor: {:?}", err),
            }
        };
        esp_println::dbg!(adc_reading);
        // ctx.send_message(wrap_message(LightLevel(lux).into_message()));
    }

    fn is_finished(&self) -> bool {
        false
    }
}
