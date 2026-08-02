extern crate alloc;
use ember::agent::bdi::{bdi_actions, bdi_agent};

#[bdi_agent(asl = {
    +!start <- .at(500, check_again); .log("info", "scheduled").

    +!check_again <- .log("info", "fired").
})]
pub struct Agent;

#[bdi_actions]
impl Agent {}

fn main() {
    let _ = Agent.into_agent();
}
