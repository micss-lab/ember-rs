extern crate alloc;
use ember::agent::bdi::{bdi_actions, bdi_agent};

#[bdi_agent(asl = {
    at(agent, home).
    !go_to(kitchen).
    +!go_to(Dest) : at(agent, Dest)
        <- .log("info", "Already there.").
    +!go_to(Dest) : at(agent, From)
        <- .log("info", "Moving.").
})]
struct ContextAgent;

#[bdi_actions]
impl ContextAgent {}

fn main() {
    let _ = ContextAgent.into_agent();
}
