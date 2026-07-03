extern crate alloc;
use ember::agent::bdi::{bdi_actions, bdi_agent};

#[bdi_agent(asl = {
    +!check : not busy <- .log("info", "not busy").
    +!check : not ready & not waiting <- .log("info", "neither ready nor waiting").
})]
struct Agent;

#[bdi_actions]
impl Agent {}

fn main() {
    let _ = Agent.into_agent();
}
