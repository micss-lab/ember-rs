extern crate alloc;
use ember::agent::bdi::{bdi_actions, bdi_agent};

// The `.send` builtin also accepts a bound variable as the receiver aid: instead of a literal
// "name@host" string validated at compile time, the address is resolved from the bindings at
// runtime (see `VariableOrReceiver`).
#[bdi_agent(asl = {
    +register(Addr)
      <- .send(Addr, "inform", ack).
})]
struct Agent;

#[bdi_actions]
impl Agent {}

fn main() {
    let _ = Agent.into_agent();
}
