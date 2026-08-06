extern crate alloc;
use ember::agent::bdi::{bdi_actions, bdi_agent};

// The `.send` builtin also accepts a literal "name@platform" aid, validated at compile time,
// instead of a variable resolved from the bindings at runtime.
#[bdi_agent(asl = {
    +!register <- .send("registry@local", "inform", ack).
})]
pub struct Agent;

#[bdi_actions]
impl Agent {}

fn main() {
    let _ = Agent.into_agent();
}
