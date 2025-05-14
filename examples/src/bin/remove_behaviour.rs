#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

use no_std_framework_examples::setup_example;

setup_example!();

use no_std_framework_core::behaviour::{BehaviourId, Context, CyclicBehaviour, OneShotBehaviour};
use no_std_framework_core::{Agent, Container};

const MESSAGE_AMOUNT: usize = 10;

struct InformationPrinter;

impl OneShotBehaviour for InformationPrinter {
    type Event = ();

    fn action(&self, _: &mut Context<Self::Event>) {
        log::info!("This agent has two behaviours.");
        log::info!("One will print infinitely, the other will stop the first after {MESSAGE_AMOUNT} iterations.")
    }
}

struct MessagePrinter;

impl CyclicBehaviour for MessagePrinter {
    type Event = ();

    fn action(&mut self, _: &mut Context<Self::Event>) {
        log::info!("Hello there!");
    }

    fn is_finished(&self) -> bool {
        false
    }
}

struct MessagePrinterStopper {
    behaviour: BehaviourId,
    count: usize,
}

impl MessagePrinterStopper {
    fn new(behaviour: BehaviourId, count: usize) -> Self {
        Self { behaviour, count }
    }
}

impl CyclicBehaviour for MessagePrinterStopper {
    type Event = ();

    fn action(&mut self, ctx: &mut Context<Self::Event>) {
        self.count -= 1;
        if self.is_finished() {
            ctx.remove_behaviour(self.behaviour);
        }
    }

    fn is_finished(&self) -> bool {
        self.count == 0
    }
}

fn example() {
    let mut agent = Agent::new("messaging-agent").with_behaviour(InformationPrinter);
    let behavour_id = agent.add_behaviour(MessagePrinter);
    let agent = agent.with_behaviour(MessagePrinterStopper::new(behavour_id, MESSAGE_AMOUNT));
    let container = Container::default().with_agent(agent);
    container.start().unwrap();
}
