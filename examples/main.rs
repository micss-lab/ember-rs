#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::prelude::*;

use no_std_framework::hardware::init_heap;
use no_std_framework::{Agent, Container};

#[entry]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();
    init_heap();

    let container = Container::default()
        .with_agent(Agent::new("agent-0"))
        .with_agent(Agent::new("agent-1"));
    container.start().unwrap();

    panic!("End of program");
}
