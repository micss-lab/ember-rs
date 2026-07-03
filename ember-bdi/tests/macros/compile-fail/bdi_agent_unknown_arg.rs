extern crate alloc;
use ember::agent::bdi::bdi_agent;

#[bdi_agent(asl = {}, foo = bar)]
struct Agent;

fn main() {}
