#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

use ember_examples::setup_example;

setup_example!();

use ember::behaviour::{Context, CyclicBehaviour, OneShotBehaviour};
use ember::{Agent, Container};

const MESSAGE_AMOUNT: usize = 10;

struct InformationPrinter;

impl OneShotBehaviour for InformationPrinter {
    type AgentState = ();

    type Event = ();

    fn action(&self, _: &mut Context<Self::Event>, _: &mut Self::AgentState) {
        log::info!("This agent has one behaviour.");
        log::info!(
            "It will seemingly print for an infinite amount of time, though it removes the agent after {MESSAGE_AMOUNT} of messages"
        );
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
        log::info!("Hello there!");
        self.count -= 1;
        if self.count == 0 {
            ctx.remove_agent();
        }
    }

    fn is_finished(&self) -> bool {
        false
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
