#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

use ember_examples::setup_example;

setup_example!();

use ember_core::behaviour::{Context, CyclicBehaviour, OneShotBehaviour};
use ember_core::{Agent, Container};

struct InformationPrinter;

impl OneShotBehaviour for InformationPrinter {
    type AgentState = ();

    type Event = ();

    fn action(&self, _: &mut Context<Self::Event>, _: &mut Self::AgentState) {
        log::info!("This agent has one behaviour.");
        log::info!("It will seemingly print for an infinite amount of time,");
        log::info!(
            "though it is blocked after the first time it requests a message and does not get one."
        );
    }
}

struct MessageChecker;

impl CyclicBehaviour for MessageChecker {
    type AgentState = ();

    type Event = ();

    fn action(&mut self, ctx: &mut Context<Self::Event>, _: &mut Self::AgentState) {
        log::info!("Checking for messages");
        ctx.block_behaviour();
    }

    fn is_finished(&self) -> bool {
        false
    }
}

fn example() {
    let container = Container::default().with_agent(
        Agent::new("messaging-agent", ())
            .with_behaviour(InformationPrinter)
            .with_behaviour(MessageChecker),
    );
    container.start().unwrap();
}
