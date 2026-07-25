extern crate alloc;
use ember::agent::bdi::{bdi_actions, bdi_agent};

#[bdi_agent(asl = {
    item(a).
    item(b).

    +!start <- .forall(item(X), process(X)).

    +!process(X) <- .log("info", "processing").
})]
struct Agent;

#[bdi_actions]
impl Agent {}

fn main() {
    let _ = Agent.into_agent();
}
