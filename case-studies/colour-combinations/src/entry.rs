use colour::Colour;
use esp_backtrace as _;

use ember::Container;
use esp_hal::clock::CpuClock;

const HEAP_SIZE: usize = 72 * 1024;

const SEQUENCE: [Colour; 13] = [
    Colour::Blue,
    Colour::Red,
    Colour::Red,
    Colour::Green,
    Colour::Green,
    Colour::Blue,
    Colour::Green,
    Colour::Red,
    Colour::Blue,
    Colour::Red,
    Colour::Blue,
    Colour::Blue,
    Colour::Blue,
];

mod colour;

pub(crate) fn main() {
    // Set newline mode to linux line endings.
    esp_println::print!("\x1b[20h");
    esp_println::logger::init_logger_from_env();
    esp_alloc::heap_allocator!(HEAP_SIZE);

    log::info!("Running case study `colour-combinations`.");

    let _peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });

    log::trace!("Initialized peripherals.");

    Container::default()
        .with_agent(colour::colour_agent(SEQUENCE))
        .start()
        .unwrap()
}
