#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::prelude::*;

use no_std_framework::behaviour::{Context, OneShotBehaviour};
use no_std_framework::hardware::init_heap;
use no_std_framework::{Agent, Container};

fn hello_world(_: &mut Context, _: ()) {
    log::info!("Hello, World!");
}

#[entry]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();
    init_heap();

    let container = Container::default()
        .with_agent(
            Agent::new("hello-world-agent").with_behaviour(OneShotBehaviour::new(hello_world)),
        )
        .with_agent(
            Agent::new("responder-agent").with_behaviour(OneShotBehaviour::new(|ctx, _| {
                log::info!("I am good!");
                ctx.stop()
            })),
        );
    container.start().unwrap();

    panic!("End of program");
}
