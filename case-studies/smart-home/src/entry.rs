use alloc::rc::Rc;
use core::cell::RefCell;

use esp_backtrace as _;
use esp_hal_embassy as _;

use esp_hal::{
    analog::adc::{Adc, AdcConfig, Attenuation},
    clock::CpuClock,
    gpio::{Input, Level, Output, Pull},
    uart::UartRx,
};
use no_std_framework_core::Container;

use home_automation::{fan, lock, pir};
use plant_monitoring::{light, moist, pump};

const HEAP_SIZE: usize = 72 * 1024;

const MOISTURE_THRESHOLD: f32 = 60.0;
const FAN_TEMPERATURE_THRESHOLD: f32 = -1.0;

const TEMP_SENSOR_VCC_VOLTAGE: f32 = 3.3;

const LOCK_PASSWORD: &[u8] = b"1234";

mod control;
mod temp;
mod utils;

pub fn main() {
    // Set newline mode to linux line endings.
    esp_println::print!("\x1b[20h");
    esp_println::logger::init_logger_from_env();
    esp_alloc::heap_allocator!(HEAP_SIZE);

    log::info!("Running case study `smart-home`.");

    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });

    log::trace!("Initialized peripherals");

    let mut adc_config = AdcConfig::new();

    // Plant monitoring system.
    let ldr_sensor = adc_config.enable_pin(peripherals.GPIO26, Attenuation::Attenuation6dB);
    let potentiometer = adc_config.enable_pin(peripherals.GPIO27, Attenuation::Attenuation6dB);
    let pump_switch = Input::new(peripherals.GPIO13, Pull::Up);
    let low_light_led = Output::new(peripherals.GPIO14, Level::Low);
    let pump_active_led = Output::new(peripherals.GPIO12, Level::Low);

    // Home automation.
    let unlock_door_switch = Input::new(peripherals.GPIO22, Pull::Up);
    let pir_sensor = Input::new(peripherals.GPIO18, Pull::Up);
    let fan_active_led = Output::new(peripherals.GPIO2, Level::Low);

    let temperature_sensor = adc_config.enable_pin(peripherals.GPIO15, Attenuation::Attenuation6dB);

    let adc = Rc::new(RefCell::new(Adc::new(peripherals.ADC2, adc_config)));

    let serial_input = UartRx::new(peripherals.UART1, peripherals.GPIO3).unwrap();

    Container::default()
        .with_agent(moist::moisture_agent(potentiometer, adc.clone()))
        .with_agent(light::light_agent(ldr_sensor, adc.clone(), low_light_led))
        .with_agent(temp::temperature_agent(temperature_sensor, adc.clone()))
        .with_agent(pump::pump_agent(pump_active_led))
        .with_agent(lock::lock_agent(
            LOCK_PASSWORD,
            unlock_door_switch,
            serial_input,
        ))
        .with_agent(fan::fan_agent())
        .with_agent(pir::pir_agent(pir_sensor))
        .with_agent(control::control_agent(pump_switch, fan_active_led))
        .start()
        .unwrap()
}
