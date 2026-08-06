extern crate alloc;
use ember::agent::bdi::{bdi_actions, bdi_agent};

#[bdi_agent(asl = {
    +!start <- .me(Name); .log("info", "identified myself").
})]
pub struct Agent;

#[bdi_actions]
impl Agent {}

fn main() {
    let _ = Agent.into_agent();
}
