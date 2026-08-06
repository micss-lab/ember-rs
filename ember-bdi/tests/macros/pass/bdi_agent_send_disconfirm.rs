extern crate alloc;
use ember::agent::bdi::{bdi_actions, bdi_agent};

// The `.send` builtin's second argument selects the performative: "inform" for an addition and
// "disconfirm" for a deletion.
#[bdi_agent(asl = {
    -active <- .send("watcher@local", "disconfirm", active).
})]
pub struct Agent;

#[bdi_actions]
impl Agent {}

fn main() {
    let _ = Agent.into_agent();
}
