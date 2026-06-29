extern crate alloc;
use ember::agent::bdi::{bdi_actions, bdi_agent};

#[bdi_agent(asl = {
    parent(tom, bob).
    parent(tom, liz).
    parent(bob, ann).
    ancestor(X, Y) :- parent(X, Y).
    ancestor(X, Y) :- parent(X, Z) & ancestor(Z, Y).
})]
struct Agent;

#[bdi_actions]
impl Agent {}

fn main() {
    let _ = Agent.into_agent();
}
