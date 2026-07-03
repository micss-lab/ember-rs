#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

use ember_examples::setup_example;

setup_example!();

use ember::Container;
use ember::agent::reactive::ReactiveAgent;
use ember::agent::reactive::behaviour::{Context, CyclicBehaviour};

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
        ReactiveAgent::new("agent-with-lifetime", ()).with_behaviour(Foo { s: "Hello, World!" }),
    );
    container.start().unwrap();
}
