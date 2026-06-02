#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

use ember_examples::setup_example;

setup_example!();

use ember::Container;
use ember::agent::reactive::ReactiveAgent;
use ember::agent::reactive::behaviour::{Context, OneShotBehaviour};

struct HelloWorld;

impl OneShotBehaviour for HelloWorld {
    type AgentState = ();

    type Event = ();

    fn action(&self, _: &mut Context<Self::Event>, _: &mut Self::AgentState) {
        log::info!("Hello, World!");
    }
}

struct Responder;

impl OneShotBehaviour for Responder {
    type AgentState = ();

    type Event = ();

    fn action(&self, ctx: &mut Context<Self::Event>, _: &mut Self::AgentState) {
        log::info!("I am good!");
        ctx.stop_platform()
    }
}

fn example() {
    let container = Container::default()
        .with_agent(ReactiveAgent::new("hello-world-agent", ()).with_behaviour(HelloWorld))
        .with_agent(ReactiveAgent::new("responder-agent", ()).with_behaviour(Responder));
    container.start().unwrap();
}
