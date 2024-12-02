#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::prelude::*;

#[entry]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();

    log::info!("Hello, World!");

    panic!("End of program");
}
