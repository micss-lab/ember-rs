#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

use no_std_framework_examples::setup_example;

setup_example!();

use no_std_framework_core::behaviour::{Context, CyclicBehaviour, OneShotBehaviour};
use no_std_framework_core::{Agent, Container};

struct InformationPrinter;

impl OneShotBehaviour for InformationPrinter {
    type Message = ();

    fn action(&self, _: &mut Context<Self::Message>) {
        log::info!("This agent has one behaviour.");
        log::info!("It will seemingly print for an infinite amount of time,");
        log::info!(
            "though it is blocked after the first time it requests a message and does not get one."
        );
    }
}

struct MessageChecker;

impl CyclicBehaviour for MessageChecker {
    type Message = ();

    fn action(&mut self, ctx: &mut Context<Self::Message>) {
        log::info!("Checking for messages");
        ctx.block_behaviour();
    }

    fn is_finished(&self) -> bool {
        false
    }
}

fn example() {
    let container = Container::default().with_agent(
        Agent::new("messaging-agent")
            .with_behaviour(InformationPrinter)
            .with_behaviour(MessageChecker),
    );
    container.start().unwrap();
}
