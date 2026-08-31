extern crate alloc;
use ember::agent::bdi::{bdi_actions, bdi_agent};

// A variable message on its own, with no callback list at all - the base case the earlier
// `bdi_agent_send_with_variable_message` test always combines with a callback.
#[bdi_agent(asl = {
    +notify(Msg) <- .send("watcher@local", "inform", Msg).
})]
pub struct Agent;

#[bdi_actions]
impl Agent {}

fn main() {
    let _ = Agent.into_agent();
}
