extern crate alloc;
use ember::agent::bdi::{bdi_actions, bdi_agent};

// A variable message on a deletion-triggered plan ("disconfirm"), not just the default
// addition/"inform" case.
#[bdi_agent(asl = {
    -active(Msg) <- .send("watcher@local", "disconfirm", Msg).
})]
pub struct Agent;

#[bdi_actions]
impl Agent {}

fn main() {
    let _ = Agent.into_agent();
}
