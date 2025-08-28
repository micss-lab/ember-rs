#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

use ember::{
    Agent, Container,
    behaviour::{Context, CyclicBehaviour},
};
use ember_examples::setup_example;

setup_example!();

struct Foo<'a> {
    s: &'a str,
}

impl CyclicBehaviour for Foo<'_> {
    type AgentState = ();

    type Event = ();

    fn action(&mut self, _: &mut Context<Self::Event>, _: &mut Self::AgentState) {
        log::info!(
            "This agent is printing something with a lifetime: {}",
            self.s
        );
    }

    fn is_finished(&self) -> bool {
        true
    }
}

fn example() {
    let container = Container::default().with_agent(
        Agent::new("agent-with-lifetime", ()).with_behaviour(Foo { s: "Hello, World!" }),
    );
    container.start().unwrap();
}
