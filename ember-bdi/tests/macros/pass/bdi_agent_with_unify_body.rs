extern crate alloc;
use ember::agent::bdi::{bdi_actions, bdi_agent};

#[bdi_agent(asl = {
    +!compute <- X = 2 + 3; .log("info", "computed").
    +!check(A, B) <- A = B; .log("info", "matched").
    +!combine(N) <- Sum = N + 1; Doubled = Sum * 2; .log("info", "combined").
})]
pub struct Agent;

#[bdi_actions]
impl Agent {}

fn main() {
    let _ = Agent.into_agent();
}
