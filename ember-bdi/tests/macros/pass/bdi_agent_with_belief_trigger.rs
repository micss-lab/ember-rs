extern crate alloc;
use ember::agent::bdi::{bdi_actions, bdi_agent};

#[bdi_agent(asl = {
    +temp(X) <- .log("info", "temperature added", X).
    -temp(X) <- .log("info", "temperature removed", X).
})]
struct Agent;

#[bdi_actions]
impl Agent {}

fn main() {
    let _ = Agent.into_agent();
}
