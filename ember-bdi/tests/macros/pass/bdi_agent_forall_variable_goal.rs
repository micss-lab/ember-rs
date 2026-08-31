extern crate alloc;
use ember::agent::bdi::{bdi_actions, bdi_agent};

// `.forall`'s goal, like `.send`'s message, may be a bound variable resolved per solution at
// runtime instead of literal ASL syntax fixed at parse time.
#[bdi_agent(asl = {
    +!process_all(Goal) <- .forall(item(X), Goal).
})]
pub struct Agent;

#[bdi_actions]
impl Agent {}

fn main() {
    let _ = Agent.into_agent();
}
