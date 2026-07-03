extern crate alloc;
use ember::agent::bdi::{bdi_actions, bdi_agent};

#[bdi_agent(asl = {
    !start.
})]
struct GoalAgent;

#[bdi_actions]
impl GoalAgent {}

fn main() {
    let _ = GoalAgent.into_agent();
}
