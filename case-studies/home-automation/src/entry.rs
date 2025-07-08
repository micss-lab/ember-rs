use esp_backtrace as _;
use esp_hal_embassy as _;

use no_std_framework_core::Container;

use self::dht22::Measurement;

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

mod control;
mod dht22;
mod fan;
mod util;

pub fn main() {
    // Set newline mode to linux line endings.
    esp_println::print!("\x1b[20h");
    esp_println::logger::init_logger_from_env();
    esp_alloc::heap_allocator!(HEAP_SIZE);

    log::info!("Running case study `home-automation`.");

    Container::default()
        .with_agent(fan::fan_agent())
        .with_agent(dht22::dht22_agent(MEASUREMENTS.into_iter().cycle()))
        .with_agent(control::control_agent())
        .start()
        .unwrap()
}
