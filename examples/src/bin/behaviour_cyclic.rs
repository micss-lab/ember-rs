#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

use no_std_framework_examples::setup_example;

setup_example!();

use no_std_framework_core::behaviour::{Context, CyclicBehaviour, OneShotBehaviour};
use no_std_framework_core::{Agent, Container};

const MESSAGE_AMOUNT: usize = 10;

struct InformationPrinter;

impl OneShotBehaviour for InformationPrinter {
    type AgentState = ();

    type Event = ();

    fn action(&self, _: &mut Context<Self::Event>, _: &mut Self::AgentState) {
        log::info!("This is the cyclic agent.");
        log::info!("I will print a message {MESSAGE_AMOUNT} times");
    }
}

struct MessagePrinter {
    count: usize,
}

impl MessagePrinter {
    fn new(count: usize) -> Self {
        Self { count }
    }
}

impl CyclicBehaviour for MessagePrinter {
    type AgentState = ();

    type Event = ();

    fn action(&mut self, ctx: &mut Context<Self::Event>, _: &mut Self::AgentState) {
        self.count -= 1;
        log::info!("Hello there!");
        if self.is_finished() {
            // Stop the container instead of only finishing this behaviour.
            ctx.stop_container()
        }
    }

    fn is_finished(&self) -> bool {
        self.count == 0
    }
}

fn example() {
    let container = Container::default().with_agent(
        Agent::new("messaging-agent", ())
            .with_behaviour(InformationPrinter)
            .with_behaviour(MessagePrinter::new(MESSAGE_AMOUNT)),
    );
    container.start().unwrap();
}
