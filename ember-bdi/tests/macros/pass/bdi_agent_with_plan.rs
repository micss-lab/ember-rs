extern crate alloc;
use ember::agent::bdi::{bdi_actions, bdi_agent};

#[bdi_agent(asl = {
    !start.
    +!start <- .log("info", "Agent started.").
})]
struct PlanAgent;

#[bdi_actions]
impl PlanAgent {}

fn main() {
    let _ = PlanAgent.into_agent();
}
