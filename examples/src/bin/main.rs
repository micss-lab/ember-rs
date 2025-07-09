#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

use no_std_framework_examples::setup_example;

setup_example!();

use no_std_framework_core::{
    behaviour::{Context, OneShotBehaviour},
    Agent, Container,
};

struct Stopper;

impl OneShotBehaviour for Stopper {
    type AgentState = ();

    type Event = ();

    fn action(&self, ctx: &mut Context<Self::Event>, _: &mut Self::AgentState) {
        log::info!("Stopping agent");
        ctx.remove_agent();
    }
}

fn example() {
    let container = Container::default()
        .with_agent(Agent::new("agent-0", ()).with_behaviour(Stopper))
        .with_agent(Agent::new("agent-1", ()).with_behaviour(Stopper));
    container.start().unwrap();
}
