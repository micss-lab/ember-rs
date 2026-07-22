extern crate alloc;
use ember::agent::bdi::bdi_agent;

#[bdi_agent(asl = {
    items([1, , 2]).
})]
struct Agent;

fn main() {}
