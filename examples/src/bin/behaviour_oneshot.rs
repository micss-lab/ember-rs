#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

use no_std_framework_examples::setup_example;

setup_example!();

use no_std_framework_core::behaviour::{Context, OneShotBehaviour};
use no_std_framework_core::{Agent, Container};

struct HelloWorld;

impl OneShotBehaviour for HelloWorld {
    type Message = ();

    fn action(&self, _: &mut Context<Self::Message>) {
        log::info!("Hello, World!");
    }
}

struct Responder;

impl OneShotBehaviour for Responder {
    type Message = ();

    fn action(&self, ctx: &mut Context<Self::Message>) {
        log::info!("I am good!");
        ctx.stop_container()
    }
}

fn example() {
    let container = Container::default()
        .with_agent(Agent::new("hello-world-agent").with_behaviour(HelloWorld))
        .with_agent(Agent::new("responder-agent").with_behaviour(Responder));
    container.start().unwrap();
}
