extern crate alloc;
use ember::agent::bdi::{bdi_actions, bdi_agent};

#[bdi_agent(asl = {})]
struct MinimalAgent;

#[bdi_actions]
impl MinimalAgent {}

fn main() {
    let _ = MinimalAgent.into_agent();
}
