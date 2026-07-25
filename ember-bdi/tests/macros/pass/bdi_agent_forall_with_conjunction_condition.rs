extern crate alloc;
use ember::agent::bdi::{bdi_actions, bdi_agent};

#[bdi_agent(asl = {
    item(a, 1).
    item(b, 2).

    +!start <- .forall(item(X, N) & N > 0, process(X)).

    +!process(X) <- .log("info", "processing").
})]
struct Agent;

#[bdi_actions]
impl Agent {}

fn main() {
    let _ = Agent.into_agent();
}
