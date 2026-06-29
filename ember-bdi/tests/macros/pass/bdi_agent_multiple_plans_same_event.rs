extern crate alloc;
use ember::agent::bdi::{bdi_actions, bdi_agent};

#[bdi_agent(asl = {
    +!handle : ready <- .log("info", "handling while ready").
    +!handle : waiting <- .log("info", "handling while waiting").
    +!handle <- .log("info", "fallback handler").
})]
struct Agent;

#[bdi_actions]
impl Agent {}

fn main() {
    let _ = Agent.into_agent();
}
