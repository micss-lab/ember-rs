use esp_backtrace as _;

use ember::Container;
use esp_hal::{
    clock::CpuClock,
    gpio::{Input, Pull},
    uart::UartRx,
};

use case_study_home_automation::{
    control,
    dht22::{self, Measurement},
    fan, lock, pir,
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

const LOCK_PASSWORD: &[u8] = b"1234";

pub fn main() {
    // Set newline mode to linux line endings.
    esp_println::print!("\x1b[20h");
    esp_println::logger::init_logger_from_env();
    esp_alloc::heap_allocator!(HEAP_SIZE);

    log::info!("Running case study `home-automation`.");

    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });

    let uart_rx = UartRx::new(peripherals.UART1, Default::default()).unwrap();
    let unlock_button = Input::new(peripherals.GPIO22, Pull::Up);
    let pir_pin = Input::new(peripherals.GPIO18, Pull::None);

    log::trace!("Initialized peripherals");

    Container::default()
        .with_agent(fan::fan_agent())
        .with_agent(dht22::dht22_agent(MEASUREMENTS.into_iter().cycle()))
        .with_agent(lock::lock_agent(LOCK_PASSWORD, unlock_button, uart_rx))
        .with_agent(pir::pir_agent(pir_pin))
        .with_agent(control::control_agent())
        .start()
        .unwrap()
}
