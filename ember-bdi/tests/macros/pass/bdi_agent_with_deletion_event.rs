extern crate alloc;
use ember::agent::bdi::{bdi_actions, bdi_agent};

#[bdi_agent(asl = {
    active.
    -active <- .log("info", "deactivated").
})]
struct Agent;

#[bdi_actions]
impl Agent {}

fn main() {
    let _ = Agent.into_agent();
}
