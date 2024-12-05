#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

use no_std_framework_core::{Agent, Container};

fn example() {
    let container = Container::default()
        .with_agent(Agent::new("agent-0"))
        .with_agent(Agent::new("agent-1"));
    container.start().unwrap();
}

#[cfg(target_os = "none")]
mod esp_imports {
    pub(super) use esp_backtrace as _;
    pub(super) use esp_hal::prelude::*;

    pub(super) use no_std_framework_examples::esp;
}

#[cfg(target_os = "none")]
use esp_imports::*;

#[cfg(target_os = "none")]
#[entry]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();
    esp::init_heap();

    example();

    panic!("End of program");
}

#[cfg(not(target_os = "none"))]
fn main() {
    use no_std_framework_examples::local;
    local::init_logger(log::LevelFilter::Trace);

    example();
}
