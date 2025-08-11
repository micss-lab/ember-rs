//! # Plant Monitoring Case Study.
//!
//! ## Requirements
//!
//! - [x] dht22 sensor: temperature measuring.
//! - [x] ldr sensor (photoresistor): light measuring.
//!     - [x] led
//! - [x] potentiometer: moisture level
//! - [ ] water pump
//!     - [x] led
//!     - [ ] buzzer
//!     - [x] user switch: enable the pump manually

use alloc::rc::Rc;
use core::cell::RefCell;

use esp_backtrace as _;
use esp_hal_embassy as _;

use esp_hal::{
    analog::adc::{Adc, AdcConfig, Attenuation},
    clock::CpuClock,
    gpio::{Input, Level, Output, Pull},
};
use ember_core::Container;

use case_study_plant_monitoring::{
    control, light, moist, pump,
    temp::{self, Measurement},
};

const HEAP_SIZE: usize = 72 * 1024;

const MEASUREMENTS: [Measurement; 10] = [
    Measurement {
        temperature: 20.0,
        humidity: 50.0,
    },
    Measurement {
        temperature: 22.0,
        humidity: 55.0,
    },
    Measurement {
        temperature: 24.0,
        humidity: 60.0,
    },
    Measurement {
        temperature: 26.0,
        humidity: 65.0,
    },
    Measurement {
        temperature: 28.0,
        humidity: 70.0,
    },
    Measurement {
        temperature: 26.0,
        humidity: 65.0,
    },
    Measurement {
        temperature: 24.0,
        humidity: 60.0,
    },
    Measurement {
        temperature: 22.0,
        humidity: 55.0,
    },
    Measurement {
        temperature: 20.0,
        humidity: 50.0,
    },
    Measurement {
        temperature: 18.0,
        humidity: 45.0,
    },
];

pub fn main() {
    // Set newline mode to linux line endings.
    esp_println::print!("\x1b[20h");
    esp_println::logger::init_logger_from_env();
    esp_alloc::heap_allocator!(HEAP_SIZE);

    log::info!("Running case study `plant-monitoring`.");

    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });

    let mut adc_config = AdcConfig::new();

    let ldr_sensor_pin = adc_config.enable_pin(peripherals.GPIO26, Attenuation::_11dB);
    let potentiometer_sensor_pin = adc_config.enable_pin(peripherals.GPIO27, Attenuation::_11dB);
    let light_alert_pin = Output::new(peripherals.GPIO4, Level::Low);
    let pump_light = Output::new(peripherals.GPIO17, Level::Low);
    let user_switch = Input::new(peripherals.GPIO15, Pull::Up);

    let adc = Rc::new(RefCell::new(Adc::new(peripherals.ADC2, adc_config)));

    log::trace!("Initialized peripherals");

    Container::default()
        .with_agent(temp::temperature_agent(MEASUREMENTS.into_iter().cycle()))
        .with_agent(light::light_agent(
            ldr_sensor_pin,
            adc.clone(),
            light_alert_pin,
        ))
        .with_agent(moist::moisture_agent(potentiometer_sensor_pin, adc))
        .with_agent(pump::pump_agent(pump_light))
        .with_agent(control::control_agent(user_switch))
        .start()
        .unwrap()
}
