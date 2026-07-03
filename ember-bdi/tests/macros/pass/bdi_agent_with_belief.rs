extern crate alloc;
use ember::agent::bdi::{bdi_actions, bdi_agent};

#[bdi_agent(asl = {
    at(agent, home).
    at(coffee_machine, kitchen).
})]
struct BeliefAgent;

#[bdi_actions]
impl BeliefAgent {}

fn main() {
    let _ = BeliefAgent.into_agent();
}
