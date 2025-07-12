use core::cell::RefCell;

use alloc::{borrow::Cow, rc::Rc};
use esp_hal::analog::adc::{Adc, AdcChannel, AdcPin, RegisterAccess};
use no_std_framework_core::{
    behaviour::{Context, TickerBehaviour},
    Agent,
};

use super::notif::{ThresholdConfig, ThresholdNotification};

pub fn moisture_agent<P: AdcChannel + 'static, ADCI: RegisterAccess>(
    potentiometer_sensor_pin: AdcPin<P, ADCI>,
    adc: Rc<RefCell<Adc<'static, ADCI>>>,
) -> Agent<MoistureState, ()> {
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

struct PotentiometerSensor<P, ADCI: 'static> {
    pin: AdcPin<P, ADCI>,
    adc: Rc<RefCell<Adc<'static, ADCI>>>,
}

impl<P, ADCI> PotentiometerSensor<P, ADCI> {
    fn new(pin: AdcPin<P, ADCI>, adc: Rc<RefCell<Adc<'static, ADCI>>>) -> Self {
        Self { pin, adc }
    }
}

impl<P, ADCI> TickerBehaviour for PotentiometerSensor<P, ADCI>
where
    P: AdcChannel,
    ADCI: RegisterAccess,
{
    type AgentState = MoistureState;

    type Event = ();

    fn interval(&self) -> core::time::Duration {
        core::time::Duration::from_millis(100)
    }

    fn action(&mut self, _: &mut Context<Self::Event>, state: &mut Self::AgentState) {
        let moisture = loop {
            match self.adc.borrow_mut().read_oneshot(&mut self.pin) {
                Ok(r) => break r,
                Err(esp_hal::prelude::nb::Error::WouldBlock) => continue,
                Err(err) => panic!("failed to read analog sensor: {:?}", err),
            }
        };
        let percent = f32::from(moisture) / 4095.0 * 100.0;
        log::debug!("moisture percent: {}", percent);
        state.percent = percent;
    }

    fn is_finished(&self) -> bool {
        false
    }
}
