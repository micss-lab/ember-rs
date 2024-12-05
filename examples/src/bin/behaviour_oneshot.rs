#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

use no_std_framework_examples::setup_example;

setup_example!();

use no_std_framework_core::behaviour::{Context, OneShotBehaviour};
use no_std_framework_core::{Agent, Container};

fn example() {
    fn hello_world(_: &mut Context, _: ()) {
        log::info!("Hello, World!");
    }

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
}
