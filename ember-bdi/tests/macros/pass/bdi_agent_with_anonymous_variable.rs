extern crate alloc;
use ember::agent::bdi::{bdi_actions, bdi_agent};

// `_` is the anonymous variable: every occurrence is independent, so `pair(_, _)` matches
// `pair(1, 2)` even though the two arguments differ (unlike `pair(X, X)`, which would require
// them to be equal). `_` is also usable in rule heads/bodies and goal arguments, wherever a term
// must be present but its value is irrelevant.
#[bdi_agent(asl = {
    pair(1, 2).
    has_pair :- pair(_, _).

    !check.
    +!check : pair(_, _)
      <- .log("info", "found a pair").
})]
struct Agent;

#[bdi_actions]
impl Agent {}

fn main() {
    let _ = Agent.into_agent();
}
