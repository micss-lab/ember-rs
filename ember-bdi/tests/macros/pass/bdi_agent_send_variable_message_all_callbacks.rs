extern crate alloc;
use ember::agent::bdi::{bdi_actions, bdi_agent};

// All four callback kinds attached to a variable message, not just `on_success` - `on_retry`
// takes a distinct codegen path (`FnMut`, cloned per attempt) from the other three (`FnOnce`).
#[bdi_agent(asl = {
    +notify(Msg)
      <- .send(
          "watcher@local", "inform", Msg,
          [on_success(ack), on_retry(retrying), on_failure(nack), on_complete(done)]
      ).
})]
pub struct Agent;

#[bdi_actions]
impl Agent {}

fn main() {
    let _ = Agent.into_agent();
}
