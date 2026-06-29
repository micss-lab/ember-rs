extern crate alloc;
use ember::agent::bdi::{bdi_actions, bdi_agent};

#[bdi_agent(asl = {
    at(agent, home).
    !move_out.
    +!move_out : at(agent, From)
        <- -at(agent, From);
           +at(agent, outside);
           !arrived.
    +!arrived <- .log("info", "Arrived!").
})]
struct BodyAgent;

#[bdi_actions]
impl BodyAgent {}

fn main() {
    let _ = BodyAgent.into_agent();
}
