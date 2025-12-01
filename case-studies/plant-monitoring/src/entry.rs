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

#[cfg(feature = "ember-based")]
use {
    alloc::rc::Rc,
    core::cell::RefCell,
};


use esp_backtrace as _;

use esp_hal::{
    analog::adc::{Adc, AdcConfig, Attenuation},
    clock::CpuClock,
    gpio::{Input, Level, Output, Pull},
};

use case_study_plant_monitoring::Measurement;

cfg_if::cfg_if! {
    if #[cfg(feature = "ember-based")] {
        use ember::Container;
        use case_study_plant_monitoring::{
            control, light, moist, pump, temp,
        };
    } else {
        use case_study_plant_monitoring::without_ember;
    }
}

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
    let peripheral_start = esp_hal::time::now();

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

    let adc = Adc::new(peripherals.ADC2, adc_config);

    log::trace!("Initialized peripherals");

    let setup_time = (esp_hal::time::now() - peripheral_start).to_nanos();
    log::debug!("peripheral start: {setup_time} ns");

    cfg_if::cfg_if! {
        if #[cfg(feature = "ember-based")] {
            let ember_start = esp_hal::time::now();

            let adc = Rc::new(RefCell::new(adc));

            let mut container = Container::default()
                .with_agent(temp::temperature_agent(MEASUREMENTS.into_iter().cycle()))
                .with_agent(light::light_agent(
                    ldr_sensor_pin,
                    adc.clone(),
                    light_alert_pin,
                ))
                .with_agent(moist::moisture_agent(potentiometer_sensor_pin, adc))
                .with_agent(pump::pump_agent(pump_light))
                .with_agent(control::control_agent(user_switch));

            log::debug!("Ember setup: {} ns", (esp_hal::time::now() - ember_start).to_nanos());
            let mut ticks = 0;
            let mut last_print = esp_hal::time::now();
            loop {
                let should_stop = container.poll().unwrap();
                if should_stop {
                    break;
                }

                ticks += 1;
                if (esp_hal::time::now() - last_print).to_secs() >= 1 {
                    log::debug!("Tps: {}", ticks);
                    ticks = 0;
                    last_print = esp_hal::time::now();
                }
            }
        } else {
            let mut measurements = MEASUREMENTS.into_iter().cycle();

            let mut adc = adc;
            let mut ldr_sensor_pin = ldr_sensor_pin;
            let mut potentiometer_sensor_pin = potentiometer_sensor_pin;
            let mut pump_light = pump_light;
            let mut light_alert_pin = light_alert_pin;
            let mut user_switch = user_switch;

            let mut last_print_time = esp_hal::time::now();

            let mut notification_checker = without_ember::NotificationChecker::default();

            let mut ticks = 0;

            loop {
                ticks += 1;

                // float temperature = dht.readTemperature();
                // float humidity = dht.readHumidity();
                let Measurement {temperature, humidity} = measurements.next().expect("program cannot continue without dht measurements");

                // int rawLight = analogRead(LDR_PIN);
                // float sensorLux = ((4095 - rawLight) / 4095.0) * (MAX_LUX - MIN_LUX) + MIN_LUX;
                // int mappedLuxGauge = (int)(((sensorLux - MIN_LUX) / (MAX_LUX - MIN_LUX)) * 4095);
                let light_lux = without_ember::read_light_lux(&mut adc, &mut ldr_sensor_pin);

                // int rawMoisture = analogRead(POTENTIOMETER_PIN);
                // int mappedMoistureLevel = map(rawMoisture, 0, 4095, 0, 100);
                let raw_moisture = match nb::block!(adc.read_oneshot(&mut potentiometer_sensor_pin)) {
                    Ok(r) => r,
                    Err(err) => panic!("failed to read analog sensor: {:?}", err),
                };
                let moisture = f32::from(raw_moisture) / 4095.0 * 100.0;

                // static unsigned long lastPrintTime = 0;
                // if (millis() - lastPrintTime >= 1000) {
                //   printSensorValues(temperature, humidity, mappedLuxGauge, rawMoisture);
                //   lastPrintTime = millis();
                // }
                if (esp_hal::time::now() - last_print_time).to_secs() >= 1 {
                    without_ember::print_sensor_values(temperature, humidity, light_lux, moisture);
                    log::debug!("Tps: {}", ticks);
                    ticks = 0;
                    last_print_time = esp_hal::time::now();
                }

                // handleLightAlert(mappedLuxGauge);
                // handlePumpControl(effectivePumpSwitch, rawMoisture);
                without_ember::handle_light_alert(light_lux, &mut light_alert_pin);
                without_ember::handle_pump_control(moisture, &mut user_switch, &mut pump_light);

                // checkLightNotification(mappedLuxGauge);
                // checkMoistureNotification(mappedMoistureLevel);
                notification_checker.check_light(light_lux);
                notification_checker.check_moisture(moisture);
            }
        }
    }
}
