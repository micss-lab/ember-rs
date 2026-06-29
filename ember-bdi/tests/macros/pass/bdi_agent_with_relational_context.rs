extern crate alloc;
use ember::agent::bdi::{bdi_actions, bdi_agent};

#[bdi_agent(asl = {
    temp(95).
    +!cool_down(X) : X > 90 <- .log("info", "too hot").
    +!heat_up(X) : X < 10 <- .log("info", "too cold").
    +!compare(X, Y) : X == Y <- .log("info", "equal").
    +!compare(X, Y) : X != Y <- .log("info", "not equal").
})]
struct Agent;

#[bdi_actions]
impl Agent {}

fn main() {
    let _ = Agent.into_agent();
}
