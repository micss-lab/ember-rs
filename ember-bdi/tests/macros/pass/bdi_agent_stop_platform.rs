extern crate alloc;
use ember::agent::bdi::{bdi_actions, bdi_agent};

#[bdi_agent(asl = {
    +!shutdown <- .stop_platform().
    +!shutdown_with_log <- .log("info", "shutting down"); .stop_platform().
})]
struct Agent;

#[bdi_actions]
impl Agent {}

fn main() {
    let _ = Agent.into_agent();
}
