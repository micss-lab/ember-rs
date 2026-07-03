extern crate alloc;
use ember::agent::bdi::{bdi_actions, bdi_agent};

#[bdi_agent(asl = {
    count(5).
    +!check(N) : N * 2 > 8 <- .log("info", "large").
    +!scale(N) : N + 1 < 10 <- .log("info", "small").
    +!diff(N) : N - 3 == 0 <- .log("info", "three").
})]
struct Agent;

#[bdi_actions]
impl Agent {}

fn main() {
    let _ = Agent.into_agent();
}
